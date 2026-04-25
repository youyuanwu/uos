use crate::dma_alloc::BootDmaAllocator;
use crate::mmio_regs::MmioRegs;
use embclox_e1000::regs::*;
use embclox_e1000::{E1000Device, RegisterAccess};
use embclox_hal_x86::pci::PciBus;

/// Reset an e1000 device to a clean post-reset state.
///
/// Disables interrupts, triggers a hardware reset, waits for completion,
/// then configures CTRL with SLU|ASDE and clears flow control registers.
/// Panics if reset doesn't complete within ~100k iterations.
pub fn reset_device(regs: &MmioRegs) {
    regs.write_reg(IMS, 0);
    let ctl = regs.read_reg(CTL);
    regs.write_reg(CTL, ctl | CTL_RST);

    let mut timeout = 100_000u32;
    loop {
        if regs.read_reg(CTL) & CTL_RST == 0 {
            break;
        }
        timeout -= 1;
        assert!(timeout > 0, "e1000 reset timeout");
    }

    regs.write_reg(IMS, 0);
    regs.write_reg(CTL, CTL_SLU | CTL_ASDE);
    regs.write_reg(FCAL, 0);
    regs.write_reg(FCAH, 0);
    regs.write_reg(FCT, 0);
    regs.write_reg(FCTTV, 0);
}

/// Create a fresh e1000 device: reset, enable bus mastering, init rings.
///
/// The device is fully reset on Drop, so callers get clean state
/// between uses.
pub fn new_device(
    regs: &MmioRegs,
    dma: &BootDmaAllocator,
) -> E1000Device<MmioRegs, BootDmaAllocator> {
    reset_device(regs);

    let pci_dev = PciBus
        .find_device_any(0x8086, &[0x100E, 0x100F, 0x10D3])
        .expect("e1000 not found on PCI bus");
    PciBus.enable_bus_mastering(&pci_dev);

    E1000Device::new(*regs, dma.clone())
}
