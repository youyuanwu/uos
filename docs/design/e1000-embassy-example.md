# Design: E1000 Embassy Networking Example

## Overview

A bare-metal TCP echo server on x86_64 QEMU using `embclox-e1000`
driver, `embclox-hal-x86` HAL, and `embclox-core` glue with
[Embassy](https://embassy.dev)'s async executor and `embassy-net`.

## Architecture

```
┌─────────────────────────────────────────────┐
│  Application (TCP echo on port 1234)        │
├─────────────────────────────────────────────┤
│  embassy-net (IP/ARP/TCP via smoltcp)       │
├─────────────────────────────────────────────┤
│  embclox-core::e1000_embassy (Driver impl)  │
├───────────────┬─────────────────────────────┤
│  embclox-e1000│  embclox-hal-x86            │
│  (driver)     │  (serial, PCI, MMIO, heap)  │
└───────────────┴─────────────────────────────┘
        ↕ MMIO (UC-mapped)      ↕ DMA (phys_offset)
┌─────────────────────────────────────────────┐
│  QEMU x86_64 q35 + e1000 NIC               │
└─────────────────────────────────────────────┘
```

## Shared crates

| Crate | Contents |
|---|---|
| `embclox-core` | `BootDmaAllocator`, `MmioRegs`, `E1000Embassy` adapter, `e1000_helpers` |
| `embclox-e1000` | Driver (RegisterAccess, DmaAllocator, E1000Device) |
| `embclox-hal-x86` | HAL (serial, PCI, MMIO, heap, timers, interrupts) |

## Project layout

```
examples/
├── .cargo/config.toml       # target = x86_64-unknown-none
├── Cargo.toml
└── src/main.rs              # boot, reset, init, executor, echo task

crates/embclox-core/         # shared glue (DMA, MMIO, Embassy adapter)
crates/embclox-e1000/        # driver (see e1000-driver-refactor.md)
crates/embclox-hal-x86/      # platform HAL (see hal-x86.md)
tools/embclox-mkimage/       # BIOS disk image builder
```

Build: `cmake -B build && cmake --build build --target image`
Test: `ctest --test-dir build`

## Future Work

- DHCP (embassy-time alarms now work)
- `embedded-io-async::Write` for serial (needs interrupt-driven UART)
