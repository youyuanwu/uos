# examples-e1000

Bare-metal kernel demonstrating Intel e1000 NIC + embassy-net TCP echo
on QEMU/KVM. Boots via `bootloader-api` (Rust-native UEFI bootloader,
no Limine).

Static IP `10.0.2.15/24` matches QEMU SLIRP defaults.

## Build & run

```bash
cd examples-e1000 && cargo build --release
```

The kernel is wrapped into a bootable image by the workspace CMake
build; see top-level `README.md`. CI runs the resulting image under
QEMU with `ctest` (no dedicated `e1000-echo` test today; the kernel is
exercised via the `qemu-tests/unit` harness).

## What this example shows

- bootloader-api `entry_point!` boot path (vs Limine in the other examples)
- PCI scan + e1000 driver init from `embclox-e1000`
- IOAPIC routing for the PCI IRQ line (vector 33)
- APIC periodic timer + spurious ISR via `embclox_hal_x86::runtime`
- Embassy executor with `hlt`-on-idle (`runtime::run_executor`)
- TCP echo on port 1234 via `embassy-net`

## Source layout

- `src/main.rs` — boot, init, ISRs, executor
