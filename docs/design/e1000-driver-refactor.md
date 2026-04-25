# Design: E1000 Driver Crate

## Overview

The `crates/embclox-e1000` crate is a clean e1000 NIC driver extracted
from the [elliott10/e1000-driver](https://github.com/elliott10/e1000-driver)
fork. `no_std`, no `alloc` dependency, log only.

## Traits

### RegisterAccess

```rust
pub trait RegisterAccess {
    fn read_reg(&self, offset: usize) -> u32;   // word index
    fn write_reg(&self, offset: usize, value: u32);
}
```

Implementations must use volatile reads/writes. `MmioRegs` in
`embclox-core` provides the standard x86 implementation.

### DmaAllocator

```rust
pub struct DmaRegion { pub vaddr: usize, pub paddr: usize, pub size: usize }

pub trait DmaAllocator {
    fn alloc_coherent(&self, size: usize, align: usize) -> DmaRegion;
    unsafe fn free_coherent(&self, region: &DmaRegion);
}
```

`free_coherent` is `unsafe` (like `Allocator::deallocate`) — caller must
ensure no references to the region exist. `E1000Device::Drop` calls it
automatically after resetting the device.

## Device API

- `new(regs, dma)` — init rings, enable TX/RX (caller must reset first)
- `mac_address()`, `link_is_up()`
- `split()` → `(RxHalf, TxHalf)` for concurrent use
- `enable_interrupts()`, `disable_interrupts()`, `handle_interrupt()`
- `enable_loopback()` — MAC internal loopback (RCTL.LBM)
- `Drop` — full device reset + free all DMA regions

## Shared helpers (`embclox-core::e1000_helpers`)

- `reset_device(regs)` — disable IRQ, CTRL_RST, wait, set SLU|ASDE
- `new_device(regs, dma)` — reset + bus mastering + `E1000Device::new`

## References

- [Intel 82540 SDM](https://pdos.csail.mit.edu/6.828/2019/readings/hardware/8254x_GBe_SDM.pdf)
