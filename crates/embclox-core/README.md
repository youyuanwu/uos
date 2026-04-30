# embclox-core

Glue code shared between the device drivers and the example
binaries.

## Modules

| Module | Purpose |
|--------|---------|
| `dma_alloc` | `BootDmaAllocator` — `DmaAllocator` impl backed by `bootloader_api`'s page mapper |
| `mmio_regs` | Generic 32-bit MMIO register accessor (`MmioRegs`) used by e1000 |
| `e1000_embassy` | `embassy_net_driver::Driver` impl for `embclox_e1000::E1000Device` |
| `e1000_helpers` | `reset_device(&regs)` — software reset sequence required before `E1000Device::new` |

This crate exists to keep the driver crates pure (no
bootloader-specific code) while still providing ready-to-use
glue for the example binaries.
