# examples-tulip

Bare-metal kernel demonstrating the DEC 21143 (Tulip) NIC +
embassy-net TCP echo on QEMU SLIRP. Boots via Limine BIOS+UEFI.

Default boot entry: `net=dhcp` (required for QEMU SLIRP CI). A
`net=static` entry exists for manual testing on the dedicated
`embclox-test` Hyper-V Internal vSwitch (`192.168.234.50/24`).

## Build & run

```bash
cmake --build build --target tulip-image    # -> build/tulip.iso
cmake --build build --target qemu-tulip     # interactive QEMU run
```

CI:

```bash
ctest --test-dir build -R tulip
# tulip-boot: waits for "TULIP INIT PASSED" log
# tulip-echo: probes TCP 5556 -> guest port 1234 returns "hello-tulip"
```

## What this example shows

- Limine boot (`limine.conf` with two cmdline-selected entries)
- HHDM-mapped DMA pool sourced from the Limine memory map
- Tulip CSR (I/O port or MMIO) access + IDT + IOAPIC routing
- Cmdline-driven DHCP/static selection via `embclox_hal_x86::cmdline`
- Embassy executor with `hlt`-on-idle (`runtime::run_executor`)
- TCP echo on port 1234

## Source layout

- `src/main.rs` — boot, PCI scan, Tulip init, IDT/IOAPIC, executor
- `src/tulip_embassy.rs` — `embassy_net_driver::Driver` impl
- `limine.conf` — boot menu entries
