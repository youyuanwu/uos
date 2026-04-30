# embclox-hal-x86

x86_64 hardware abstraction layer for the embclox example kernels.

## Modules

| Module | What |
|--------|------|
| `apic` | Local APIC (xAPIC MMIO): enable, periodic + one-shot timer, EOI |
| `ioapic` | I/O APIC: route external IRQs to LAPIC vectors |
| `pic` | Legacy 8259 PIC (only used to disable it before APIC takes over) |
| `idt` | Shared IDT singleton + `set_handler` |
| `pit` | TSC calibration via PIT channel 2 (bounded; returns `None` on Hyper-V Gen1 where PIT ch2 isn't emulated) |
| `time` | `embassy_time_driver::Driver` impl over TSC + alarm table |
| `runtime` | Shared APIC-timer ISR + executor loop + `block_on_hlt` |
| `memory` | Page-table mapper for MMIO ranges (`map_mmio`) |
| `heap` | Global heap (`linked_list_allocator::LockedHeap`) |
| `serial` | UART 16550 driver + `log` backend |
| `pci` | Type-1 PCI config-space scanner |
| `cmdline` | Bootloader-agnostic `net=dhcp` / `net=static` parser |

## Runtime API (the part most examples use)

```rust
embclox_hal_x86::idt::init();
embclox_hal_x86::pic::disable();

let lapic_vaddr = memory.map_mmio(LAPIC_PHYS_BASE, 0x1000).vaddr();
let mut lapic = LocalApic::new(lapic_vaddr);
lapic.enable();

let tsc_per_us = pit::calibrate_tsc_mhz().unwrap_or(default);
embclox_hal_x86::time::set_tsc_per_us(tsc_per_us);

embclox_hal_x86::runtime::start_apic_timer(lapic, tsc_per_us, 1_000);
// ... spawn embassy tasks ...
embclox_hal_x86::runtime::run_executor(executor);     // never returns
```

`runtime::block_on_hlt(future)` runs a single future to completion,
halting the CPU between polls — useful for boot-time async waits
before the embassy executor is up.

(No host-side tests — the global allocator prevents `cargo test`.
See `embclox-async` for testable runner logic.)
