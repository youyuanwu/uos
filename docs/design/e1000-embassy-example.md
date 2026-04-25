# Design: E1000 Embassy Networking Example

## Overview

Build an example program that uses the `e1000-driver` crate together with the
[Embassy](https://embassy.dev) async embedded framework to provide a complete,
async-capable networking stack on bare-metal x86_64 (QEMU `q35` or `pc`
machine).

The existing example in `external/e1000/_examples` demonstrates raw packet
TX/RX with a hand-crafted ARP frame on RISC-V. This design replaces the
polling loop with Embassy's async executor, targets x86_64 instead, and
plugs the e1000 driver into `embassy-net` so the application can use TCP,
UDP, DHCP, and DNS out of the box.

## Goals

| # | Goal |
|---|------|
| 1 | Wrap `e1000-driver` in an adapter that implements the `embassy-net-driver` traits |
| 2 | Run `embassy-net` (smoltcp-based) networking stack on top |
| 3 | Demonstrate a useful network task (e.g. TCP echo server, or DHCP + ping reply) |
| 4 | Target `x86_64-unknown-none` on QEMU with `-device e1000` |
| 5 | Keep the example self-contained and **not** compiled by the workspace by default (same convention as `_examples`) |

## Non-Goals

- Production-quality OS or kernel
- Multi-core / SMP support
- Virtual memory / paging (run with bootloader's offset-mapped memory)
- Supporting physical hardware initially

## E1000 Driver Modifications

Since we own the `external/e1000` code, we make targeted changes to
simplify the Embassy integration and fix existing bugs:

**1. Fix `e1000_recv()` inverted return** (`e1000_inner.rs:357-361`):
Swap `Some`/`None` so `Some(packets)` = data, `None` = nothing.

**2. Fix device reset sequence** — wait for `CTRL_RST` to clear before
writing registers, set `CTRL_SLU | CTRL_ASDE` after reset, disable
flow control registers. Without waiting for reset, register writes
are silently dropped. (Reference: Redox OS e1000d driver.)

**3. Add `e1000_recv_with()` — zero-copy callback receive**:
A new method that processes one descriptor at a time and passes the
DMA buffer directly to a caller-supplied closure. No heap allocation,
no `to_vec()` copy. Maps perfectly to Embassy's `RxToken::consume()`.

```rust
/// Receive one packet via zero-copy callback.
/// The closure `f` receives a `&mut [u8]` slice directly into the
/// DMA buffer. The descriptor is recycled after `f` returns.
pub fn e1000_recv_with<R, F: FnOnce(&mut [u8]) -> R>(&mut self, f: F) -> Option<R> {
    let rindex = (self.regs[E1000_RDT].read() as usize + 1) % RX_RING_SIZE;
    if self.rx_ring[rindex].addr == 0 { return None; }
    let status = self.rx_ring[rindex].status;
    // Require both DD (descriptor done) and EOP (end of packet)
    if (status & (E1000_RXD_STAT_DD | E1000_RXD_STAT_EOP) as u8)
        != (E1000_RXD_STAT_DD | E1000_RXD_STAT_EOP) as u8 { return None; }
    // Drop packets with RX errors
    if self.rx_ring[rindex].errors != 0 {
        self.rx_ring[rindex].status = 0;
        self.regs[E1000_RDT].write(rindex as u32);
        return None;
    }

    fence_r();  // read barrier before accessing DMA buffer contents
    let len = min(self.rx_ring[rindex].length as usize, self.mbuf_size);
    let mbuf = unsafe { from_raw_parts_mut(self.rx_mbufs[rindex] as *mut u8, len) };
    let result = f(mbuf);

    fence();
    mbuf[..min(64, len)].fill(0);
    self.rx_ring[rindex].status = 0;
    self.regs[E1000_RDT].write(rindex as u32);
    self.e1000_write_flush();
    fence_w();

    Some(result)
}

/// Check if a packet is ready without consuming it.
pub fn has_rx_packet(&self) -> bool {
    let rindex = (self.regs[E1000_RDT].read() as usize + 1) % RX_RING_SIZE;
    self.rx_ring[rindex].addr != 0
        && (self.rx_ring[rindex].status & E1000_RXD_STAT_DD as u8) != 0
}

/// Check if a TX descriptor is available.
pub fn has_tx_space(&self) -> bool {
    let tindex = self.regs[E1000_TDT].read() as usize;
    (self.tx_ring[tindex].status & E1000_TXD_STAT_DD as u8) != 0
}
```

**3. Remove `net_rx()` stub call** from `e1000_recv()` — it's a no-op
that touches packet data before the caller sees it.

**4. Keep existing `e1000_recv()` / `e1000_transmit()`** for backward
compatibility with the RISC-V `_examples`. Just fix the return value.

---

## Architecture

```
┌─────────────────────────────────────────────┐
│              Application Tasks              │
│   (TCP echo server, DHCP client, etc.)      │
├─────────────────────────────────────────────┤
│              embassy-net                    │
│   (IP, ARP, TCP, UDP, DHCP — via smoltcp)  │
├─────────────────────────────────────────────┤
│         embassy-net-driver adapter          │
│   Implements Driver / RxToken / TxToken     │
├─────────────────────────────────────────────┤
│            e1000-driver crate               │
│   E1000Device<KernelFunc>                   │
├─────────────────────────────────────────────┤
│           Platform / Boot layer             │
│   bootloader, heap, PCI I/O ports, serial   │
└─────────────────────────────────────────────┘
        ↕ MMIO / I/O ports         ↕ DMA memory
┌─────────────────────────────────────────────┐
│   QEMU x86_64 (q35/pc, e1000 NIC emulation)│
└─────────────────────────────────────────────┘
```

---

## Key Components

### 1. Boot Layer (x86_64)

Use the [`bootloader`](https://docs.rs/bootloader/) crate (v0.11+) which
handles the x86_64 boot process (real → long mode, page tables, GDT) and
passes control to a Rust entry point with a `BootInfo` struct containing
the memory map and framebuffer info.

The bootloader maps all physical memory at a configurable **offset**
(`physical_memory_offset` from `BootInfo`), not identity-mapped. All
physical-to-virtual translations must add this offset, and all
virtual-to-physical translations (needed for DMA) must subtract it.

```rust
bootloader_api::entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // init serial (0x3F8), heap, PCI scan, e1000 init
    // ...
    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(net_task(/* ... */)).unwrap();
        spawner.spawn(app_task(/* ... */)).unwrap();
    });
}
```

**Console I/O:** Use the x86 UART 16550 serial port at I/O port `0x3F8`
(COM1). QEMU's `-serial mon:stdio` connects this to the host terminal.
The [`uart_16550`](https://docs.rs/uart_16550/) crate provides a safe
wrapper.

### 2. Embassy Executor (`embassy-executor`)

The async runtime. On bare-metal x86_64 we use the spin-loop executor
(`arch-spin` feature) which busy-polls futures. The BSP core enters the
executor after platform init.

> **Polling overhead:** The `arch-spin` executor busy-polls in a tight
> loop. Each poll of `receive()` triggers an MMIO register read, which
> costs ~1-5μs per VM exit on KVM. Insert `core::hint::spin_loop()`
> (x86 `pause` instruction) in the idle path to reduce host CPU waste.
> This is acceptable for a prototype; Phase 6 (interrupts) eliminates
> the busy-poll entirely.

### 3. E1000 → Embassy Driver Adapter

With the zero-copy `e1000_recv_with()` and peek methods added to the
driver, the adapter is straightforward. It uses `UnsafeCell` to allow
both `RxToken` and `TxToken` to reference the device from a single
`receive()` call (safe because smoltcp consumes tokens sequentially).

The `receive()` and `transmit()` methods must call
`cx.waker().wake_by_ref()` when returning `None` to keep the
busy-poll executor re-scheduling the network runner task.

```rust
pub struct E1000Embassy {
    device: UnsafeCell<E1000Device<'static, Kernfn>>,
    mac: [u8; 6],
}

impl Driver for E1000Embassy {
    fn receive(&mut self, cx: &mut Context) -> Option<(RxToken, TxToken)> {
        if self.dev().has_rx_packet() && self.dev().has_tx_space() {
            return Some((RxToken { parent: self }, TxToken { parent: self }));
        }
        cx.waker().wake_by_ref(); // keep polling
        None
    }

    fn transmit(&mut self, cx: &mut Context) -> Option<TxToken> {
        if self.dev().has_tx_space() {
            return Some(TxToken { parent: self });
        }
        cx.waker().wake_by_ref();
        None
    }

    fn link_state(&mut self, cx: &mut Context) -> LinkState {
        cx.waker().wake_by_ref();
        LinkState::Up
    }
    // ...
}

impl RxToken for E1000RxToken<'_> {
    fn consume<R, F: FnOnce(&mut [u8]) -> R>(self, f: F) -> R {
        self.parent.dev_mut().e1000_recv_with(f)
            .expect("packet was ready in receive()")
    }
}

impl TxToken for E1000TxToken<'_> {
    fn consume<R, F: FnOnce(&mut [u8]) -> R>(self, len: usize, f: F) -> R {
        let mut buf = [0u8; 1514];
        let result = f(&mut buf[..len]);
        self.parent.dev_mut().e1000_transmit(&buf[..len]);
        result
    }
}
```

Key design points:
- **Zero-copy RX** — `RxToken::consume()` reads directly from the DMA
  buffer via `e1000_recv_with()`. No `Vec`, no `to_vec()`, no `.clone()`.
- **No packet buffering** — each `receive()` peeks one descriptor;
  `RxToken::consume()` processes exactly that one. No `VecDeque` needed.
- **TX availability check** — `receive()` returns `(RxToken, TxToken)`
  only when both RX data and TX space are available.
- **Waker re-scheduling** — `cx.waker().wake_by_ref()` on `None` return
  keeps the Embassy executor re-polling (required for `platform-spin`
  busy-poll mode — without this, the runner task goes to sleep forever).
- **`UnsafeCell`** — needed because `Driver::receive()` returns both
  `RxToken` and `TxToken` referencing the same device. Safe because
  smoltcp consumes tokens sequentially, not concurrently.

### 4. PCI Configuration (x86 I/O Ports)

On x86_64, PCI configuration space is accessed via I/O ports `0xCF8`
(address) and `0xCFC` (data). The existing `_examples/src/pci_impl.rs`
already has x86_64 PCI I/O port access via `x86_64::instructions::port::Port`.

We reuse the I/O port pattern with `CSpaceAccessMethod::IO` (standard PCI
config mechanism 1).

> **Important:** The e1000 crate's built-in `pci.rs` hardcodes RISC-V QEMU
> addresses (`E1000_REGS = 0x40000000`, `ECAM = 0x30000000`). These are
> **not valid on x86_64**. On QEMU q35/pc, the e1000 BAR0 is assigned by
> firmware (typically in the `0xFEBx_xxxx` range). The BAR0 address must
> be read dynamically from PCI config space after bus enumeration — do
> **not** use `e1000_driver::pci::pci_init()` or `E1000_REGS`.

The PCI scanner should:
1. Enumerate bus 0, scan for vendor `0x8086` / device `0x100E`
2. Read BAR0 from PCI config offset `0x10`
3. Enable bus mastering and memory space in the PCI command register
4. **After e1000 device reset** (in `e1000_init()`), re-enable bus
   mastering — the device reset clears the PCI command register
5. Use 16-bit writes to the PCI command register (offset `0x04`) to
   avoid corrupting the adjacent status register

```rust
use x86_64::instructions::port::Port;

impl PortOps for PortOpsImpl {
    unsafe fn read32(&self, port: u32) -> u32 {
        Port::new(port as u16).read()
    }
    unsafe fn write32(&self, port: u32, val: u32) {
        Port::new(port as u16).write(val);
    }
    // ...
}

fn pci_find_e1000(phys_offset: u64) -> Option<usize> {
    // Scan PCI bus 0 for device 8086:100E
    // Read BAR0, translate: vaddr = bar0_phys + phys_offset
    // Return vaddr for E1000Device::new()
}
```

If no e1000 device is found, log a diagnostic message and halt gracefully
rather than accessing unmapped memory.

### 5. `KernelFunc` Implementation

The `KernelFunc` trait requires `dma_alloc_coherent(pages)` to return
page-aligned `(vaddr, paddr)` pairs. The new implementation must handle
**arbitrary page counts** — the driver requests 1-page allocations for
descriptor rings and **128-page** (512 KiB) allocations for packet buffers.

> **Do not copy the existing `Kernfn` from `_examples/src/e1000.rs`.**
> It only handles 1-page and 8-page requests via hardcoded `Box::new`
> arrays. A 128-page request silently falls back to 1 page, causing
> memory corruption.

Use `Layout::from_size_align` for proper page-aligned allocation.
The returned `vaddr` must go through the bootloader's
`physical_memory_offset` mapping (not the kernel segment mapping) to
ensure DMA coherency in QEMU TCG mode:

```rust
pub struct Kernfn {
    kernel_offset: u64,  // kernel vaddr → paddr translation
    phys_offset: u64,    // bootloader's physical memory mapping offset
}

impl KernelFunc for Kernfn {
    fn dma_alloc_coherent(&mut self, pages: usize) -> (usize, usize) {
        let size = pages * Self::PAGE_SIZE;
        let layout = Layout::from_size_align(size, Self::PAGE_SIZE)
            .expect("invalid DMA layout");
        let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
        assert!(!ptr.is_null(), "DMA allocation of {} pages failed", pages);
        let heap_vaddr = ptr as usize;
        let paddr = heap_vaddr - self.kernel_offset as usize;
        // Return vaddr through phys_offset mapping for DMA coherency
        let vaddr = paddr + self.phys_offset as usize;
        (vaddr, paddr)
    }
    // ...
}
```

Key points:
- **Page alignment** is explicit via `Layout`, not reliant on `Box`
- **Physical address** = heap virtual address minus `kernel_offset`
- **DMA virtual address** = physical address plus `phys_offset` — this
  ensures CPU reads of DMA memory go through the bootloader's physical
  memory mapping, which is coherent with QEMU's DMA writes (see
  Implementation Findings below)
- **All allocation sizes** are handled (not just 1 and 8 pages)
- **Failure** panics with a diagnostic message

### 6. Networking Stack (`embassy-net`)

Start with a **static IP** configuration for Phase 4 (avoids dependency
on `embassy-time` for DHCP retransmission timers). Switch to DHCP after
the time driver is implemented in Phase 6.

```rust
let config = embassy_net::Config::ipv4_static(StaticConfigV4 {
    address: Ipv4Cidr::new(Ipv4Address::new(10, 0, 2, 15), 24),
    gateway: Some(Ipv4Address::new(10, 0, 2, 2)),
    dns_servers: Default::default(),
});

static RESOURCES: StaticCell<StackResources<5>> = StaticCell::new();
let resources = RESOURCES.init(StackResources::new());

// embassy-net 0.9 returns (Stack, Runner) from a free function
let (stack, runner) = embassy_net::new(driver, config, resources, seed);
static STACK: StaticCell<Stack> = StaticCell::new();
let stack = &*STACK.init(stack);

// Runner task drives smoltcp; must be spawned as a separate task
#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, E1000Embassy>) {
    runner.run().await;
}
```

### 7. Application Task (TCP Echo)

```rust
#[embassy_executor::task]
async fn echo_task(stack: &'static Stack<E1000EmbassyDriver>) {
    let mut socket_rx_buf = [0u8; 1024];
    let mut socket_tx_buf = [0u8; 1024];
    let mut read_buf = [0u8; 1024];

    loop {
        let mut socket = TcpSocket::new(stack, &mut socket_rx_buf, &mut socket_tx_buf);
        socket.accept(1234).await.unwrap();

        loop {
            let n = match socket.read(&mut read_buf).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            };
            socket.write_all(&read_buf[..n]).await.ok();
        }
    }
}
```

> Note: `socket_rx_buf` / `socket_tx_buf` are the socket's internal
> buffers (owned by `TcpSocket`). `read_buf` is a separate buffer for
> `socket.read()` — these must be distinct to satisfy Rust's borrow rules.

### 8. Timer / Time Driver

`embassy-time` needs a time driver implementing the `Driver` trait from
`embassy-time-driver`. For the initial busy-poll version, a minimal TSC
stub suffices:

```rust
struct TscTimeDriver;

impl Driver for TscTimeDriver {
    fn now(&self) -> u64 {
        unsafe { core::arch::x86_64::_rdtsc() / 1000 } // ~microsecond ticks
    }

    fn schedule_wake(&self, _at: u64, waker: &Waker) {
        waker.wake_by_ref(); // always wake — executor re-polls
    }
}

embassy_time_driver::time_driver_impl!(static DRIVER: TscTimeDriver = TscTimeDriver);
```

This provides monotonic time from TSC and always-wake semantics that
work with the busy-poll executor. For proper alarm-based waking
(Phase 6), replace with APIC timer interrupts.

### 9. Interrupt Handling

The e1000 device raises MSI or legacy PCI interrupts on packet arrival.
On x86_64, interrupts are delivered through the IDT. The interrupt handler
should:

1. Call `e1000_device.e1000_intr()` to acknowledge the interrupt
2. Wake the Rx waker registered by the driver adapter
3. Send EOI to the local APIC (or PIC if using legacy mode)

For the initial version, **polling mode** (no interrupts) is acceptable —
the executor loop will repeatedly call `receive()`. This matches the
existing example's approach. Interrupt-driven wake-up can be added later
by setting up the IDT and routing the e1000 IRQ.

---

## Project Layout

```
_examples_embassy/               # at repo root (not in workspace members)
    ├── .cargo/
    │   └── config.toml          # target = x86_64-unknown-none, RUSTC_BOOTSTRAP=1
    ├── Cargo.toml               # edition = "2024", [workspace] to isolate
    ├── Makefile                  # build + image + qemu targets
    ├── test.sh                  # automated QEMU boot + TCP echo test
    └── src/
        ├── critical_section_impl.rs  # x86 interrupt-based critical-section
        ├── e1000_adapter.rs     # embassy-net-driver impl (UnsafeCell-based)
        ├── heap.rs              # linked_list_allocator on static BSS
        ├── kernfn.rs            # KernelFunc with phys_offset DMA mapping
        ├── logger.rs            # log crate over serial
        ├── mmio.rs              # UC page mapping via x86_64 OffsetPageTable
        ├── pci_init.rs          # x86 I/O port PCI scanner + bus mastering
        ├── serial.rs            # UART 16550 (COM1 @ 0x3F8)
        ├── time_driver.rs       # TSC-based embassy-time stub
        └── main.rs              # bootloader entry, executor, TCP echo

tools/mkimage/                   # in workspace — builds BIOS disk images
    ├── Cargo.toml               # depends on bootloader 0.11
    └── src/main.rs              # DiskImageBuilder CLI
```

The example uses `RUSTC_BOOTSTRAP=1` in `.cargo/config.toml` to enable
unstable features on stable Rust, and an empty `[workspace]` table in
`Cargo.toml` to isolate from the parent workspace. The `mkimage` tool
lives in the workspace at `tools/mkimage/` and creates bootable BIOS
disk images.

The `bootloader` crate handles all x86_64 boot complexity (real → protected
→ long mode, GDT, page tables) so no custom assembly or linker script is
needed.

## Dependencies

```toml
[dependencies]
embassy-executor   = { version = "0.10", features = ["platform-spin", "executor-thread"] }
embassy-net        = { version = "0.9", features = ["tcp", "udp", "medium-ethernet", "proto-ipv4", "log"] }
embassy-net-driver = "0.2"
embassy-time       = { version = "0.5", features = ["tick-hz-1_000_000"] }
embassy-time-driver = "0.2"
embassy-sync       = "0.8"
embassy-futures    = "0.1"
embedded-io-async  = "0.7"

e1000-driver     = { path = "../external/e1000" }
bootloader_api   = "0.11"
x86_64           = "0.15"
uart_16550       = "0.3"
log              = "0.4"
static_cell      = "2"
linked_list_allocator = "0.10"
critical-section = { version = "1", features = ["restore-state-bool"] }
spin             = "0.9"
```

> **Toolchain:** Uses stable Rust 1.95.0 with `RUSTC_BOOTSTRAP=1` set
> in `.cargo/config.toml`. Embassy's `platform-spin` feature provides a
> busy-poll executor. The `bootloader` crate (build dependency in
> `tools/mkimage`) creates the bootable BIOS disk image.
>
> **Embassy versions:** These are the actual tested versions. The `log`
> feature on `embassy-net` enables `log` crate integration so smoltcp
> diagnostics appear on the serial console.

## QEMU Invocation

```sh
qemu-system-x86_64 \
    -machine q35 \
    -no-reboot \
    -serial mon:stdio \
    -display none \
    -drive format=raw,file=target/x86_64-unknown-none/debug/e1000-embassy-example.img \
    -netdev user,id=net0,hostfwd=tcp::5555-:1234 \
    -device e1000,netdev=net0
```

The bootable disk image is produced by `tools/mkimage` using
`bootloader::DiskImageBuilder`. Run `make image` to build the kernel
and create the disk image. The `hostfwd` flag maps host port 5555 →
guest port 1234. Test with: `echo "hello" | nc -w 3 localhost 5555`

An automated test is provided: `./test.sh` builds, boots QEMU, sends
a TCP echo request, verifies the response, and exits.

---

## Phased Implementation Plan

### Phase 1 — Scaffold & Boot

- Set up project structure, Cargo.toml, `.cargo/config.toml`
- Configure `bootloader` crate for x86_64 boot
- Initialize UART 16550 serial console, heap allocator, logger
- Verify the binary boots and prints to QEMU serial console

### Phase 2 — E1000 Driver Init + Platform Bring-up

- **Modify e1000 driver** (`external/e1000/src/e1000/e1000_inner.rs`):
  - Fix `e1000_recv()` inverted `Some`/`None` return
  - Fix reset: wait for `CTRL_RST` clear, set `SLU|ASDE`, disable FC
  - Add `e1000_recv_with()`, `has_rx_packet()` (volatile), `has_tx_space()`
  - Add `fence_r()` read barrier, check `EOP` and `errors` in recv
  - Remove `net_rx()` stub; enable promiscuous mode in RCTL
- **Map BAR0 as Uncacheable** via `x86_64::OffsetPageTable::map_to` +
  `PageTableFlags::NO_CACHE` at a fresh virtual address
- PCI scanner via x86 I/O ports (16-bit writes to command register)
- **Re-enable bus mastering after device reset**
- `KernelFunc`: page-aligned alloc, DMA vaddr through `phys_offset`
- Send gratuitous ARP after init to trigger QEMU slirp
- `critical-section` impl via x86 interrupt disable/enable

### Phase 3 — Embassy Driver Adapter

- `UnsafeCell`-based Driver impl with zero-copy `e1000_recv_with()`
- `cx.waker().wake_by_ref()` on `None` to keep executor polling
- `receive()` checks both `has_rx_packet()` and `has_tx_space()`

### Phase 4 — Embassy Executor + Networking Stack

- `embassy-executor` with `platform-spin` (busy-poll)
- `embassy-net::new()` returns `(Stack, Runner)` — runner in separate task
- Static IP (10.0.2.15/24, gateway 10.0.2.2)
- TSC-based `embassy-time-driver` stub (always-wake)
- `StackResources::<5>` for socket headroom

### Phase 5 — Application Task

- Add TCP echo server on port 1234
- Test from host via `nc localhost 5555` (through QEMU hostfwd)

### Phase 6 (Optional) — Interrupt-Driven Mode + DHCP

- Set up IDT with x86_64 crate
- Implement timer driver for `embassy-time` using TSC + APIC timer
- Switch to DHCP configuration (now that timers work for retransmission)
- Route e1000 PCI interrupt through IOAPIC → IDT
- ISR calls `e1000_intr()` directly on registers (through the
  `critical-section` Mutex) and wakes the Rx waker
- Replace polling with interrupt-driven receive

---

## Implementation Findings

Issues discovered and resolved during implementation:

### 1. MMIO Caching (Critical)

The `bootloader` crate maps **all** physical memory (including device
MMIO at BAR0 `~0xFEB80000`) using Write-Back cached 2MiB huge pages.
QEMU's TCG softmmu does not dispatch MMIO writes correctly through
WB-cached pages — writes go to QEMU's RAM emulation instead of the
e1000 device model.

**Fix:** Create a separate virtual address mapping for BAR0 using the
`x86_64` crate's `OffsetPageTable::map_to` with
`PageTableFlags::NO_CACHE`. This maps BAR0 at a fresh virtual address
(`0x4000_0000_0000`) with 4KB UC pages, bypassing the bootloader's
WB mapping entirely.

### 2. DMA Coherency in QEMU TCG (Critical)

DMA descriptor rings and packet buffers are allocated from the kernel
heap (BSS segment). The e1000 hardware writes DD (Descriptor Done) bits
and packet data to the **physical** addresses in these buffers. However,
the kernel segment's virtual addresses go through a different TLB path
than QEMU's DMA writes, causing stale reads.

**Fix:** The `KernelFunc::dma_alloc_coherent()` returns `vaddr` through
the bootloader's `physical_memory_offset` mapping (`paddr + phys_offset`)
instead of the kernel segment mapping. This ensures CPU reads of DMA
memory use the same TLB entries that QEMU's DMA engine writes through.

### 3. PCI Bus Mastering Lost After Reset (Critical)

The e1000 driver's `e1000_init()` performs a device reset via
`CTRL_RST`. This clears the PCI command register, including the bus
mastering bit. Without bus mastering, the e1000 cannot perform DMA
(both TX and RX fail silently).

**Fix:** Re-enable bus mastering in the PCI command register **after**
`E1000Device::new()` returns (after the reset completes).

### 4. Device Reset Timing (Important)

The original driver wrote to registers immediately after asserting
`CTRL_RST` without waiting for the reset to complete. The Redox OS
e1000d driver (reference implementation) waits in a loop for the
`CTRL_RST` bit to clear before proceeding.

**Fix:** Added a spin-loop after reset: `while CTRL & CTRL_RST != 0`.
Also set `CTRL_SLU | CTRL_ASDE` (Set Link Up + Auto-Speed Detection
Enable) and clear flow control registers after reset.

### 5. Executor Waker Starvation (Important)

Embassy's `Runner::run()` uses `poll_fn` which returns `Poll::Pending`
and relies on wakers to be re-scheduled. The `platform-spin` executor
busy-polls, but only re-polls tasks whose wakers have been triggered.
If `Driver::receive()` returns `None` without waking, the runner task
sleeps forever and never polls the network again.

**Fix:** Call `cx.waker().wake_by_ref()` in `receive()`, `transmit()`,
and `link_state()` when returning `None`/`Up`. This ensures the
executor continuously re-polls the network runner.

### 6. Gratuitous ARP for QEMU Slirp (Minor)

QEMU's slirp backend checks `rx_can_recv` when it has a packet to
deliver. If the check fails (during early boot before our init), slirp
caches the failure and stops retrying. A TX from the guest triggers
`qemu_flush_queued_packets()` which re-evaluates `rx_can_recv`.

**Fix:** Send a gratuitous ARP frame immediately after e1000 init
completes, before starting the Embassy executor. This forces slirp
to re-check RX readiness.

---

## Resolved Open Questions

1. **Bootloader version** — Using `bootloader_api` v0.11 with
   `BootloaderConfig::mappings.physical_memory = Some(Mapping::Dynamic)`.
   Disk images built by `tools/mkimage` using `bootloader` v0.11's
   `DiskImageBuilder`.

2. **Embassy versions** — Pinned: `embassy-executor 0.10` (feature
   `platform-spin`), `embassy-net 0.9`, `embassy-net-driver 0.2`,
   `embassy-time 0.5`, `embassy-sync 0.8`. Verified compatible.

3. **DMA coherency** — Resolved via `phys_offset` mapping (Finding #2).
   The `kernel_offset` (`0xFFFF000000` on QEMU) converts kernel heap
   addresses to physical. Physical addresses + `phys_offset` gives the
   DMA-coherent virtual address.

4. **`e1000_recv()` bug** — Fixed directly in `external/e1000`. Both
   the inverted return and the `net_rx()` stub call were fixed.

5. **Per-packet allocation** — Eliminated via `e1000_recv_with()`.

6. **Toolchain** — Stable Rust 1.95.0 with `RUSTC_BOOTSTRAP=1` in
   `.cargo/config.toml`. Edition 2024 for the example crate (required
   by `bootloader_api 0.11`). The `mkimage` tool uses edition 2021.

---

## References

- [e1000-driver crate](../external/e1000/) — the driver being wrapped
- [e1000 _examples](../external/e1000/_examples/) — existing bare-metal example (RISC-V)
- [Embassy](https://embassy.dev) — async embedded framework
- [embassy-net docs](https://docs.embassy.dev/embassy-net/) — networking stack
- [embassy-net-driver](https://docs.rs/embassy-net-driver/) — driver trait
- [bootloader crate](https://docs.rs/bootloader/) — x86_64 boot for Rust kernels
- [x86_64 crate](https://docs.rs/x86_64/) — x86_64 structures (IDT, GDT, paging, I/O ports)
- [Intel 82540 SDM](https://pdos.csail.mit.edu/6.828/2019/readings/hardware/8254x_GBe_SDM.pdf)
- [Redox OS e1000d driver](https://github.com/redox-os/drivers/blob/master/net/e1000d/src/device.rs) — reference for reset sequence and init
- [ArceOS e1000 integration](https://github.com/elliott10/arceos/blob/net-e1000/crates/driver_net/src/e1000.rs)
