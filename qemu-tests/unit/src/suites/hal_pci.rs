use embclox_hal_x86::pci::PciBus;

/// PCI bus enumeration and configuration tests.
/// PciBus is a zero-sized type — no init needed, just construct inline.
#[embclox_test_macros::test_suite(name = "hal_pci")]
mod tests {
    use super::*;

    /// Verify that PCI scan finds the e1000 NIC on the QEMU q35 bus.
    #[test]
    fn find_e1000_device() {
        let dev = PciBus
            .find_device_any(0x8086, &[0x100E, 0x100F, 0x10D3])
            .expect("e1000 should be present on QEMU q35");
        assert_eq!(dev.vendor, 0x8086);
        assert!(
            [0x100E, 0x100F, 0x10D3].contains(&dev.device),
            "unexpected device ID: {:#06x}",
            dev.device
        );
    }

    /// BAR0 should be assigned a non-zero MMIO address by QEMU firmware.
    #[test]
    fn bar0_is_nonzero() {
        let dev = PciBus
            .find_device_any(0x8086, &[0x100E, 0x100F, 0x10D3])
            .unwrap();
        let bar0 = PciBus.read_bar(&dev, 0);
        assert!(bar0 != 0, "BAR0 should be mapped to a non-zero address");
    }

    /// Enable bus mastering and verify the command register reflects it.
    #[test]
    fn bus_mastering_enable() {
        let dev = PciBus
            .find_device_any(0x8086, &[0x100E, 0x100F, 0x10D3])
            .unwrap();
        PciBus.enable_bus_mastering(&dev);
        let cmd = PciBus.read_config(&dev, 0x04) & 0xFFFF;
        assert!(
            cmd & 0x04 != 0,
            "bus mastering bit should be set, CMD={:#06x}",
            cmd
        );
    }

    /// PCI interrupt line register should contain a valid legacy IRQ (0–31).
    #[test]
    fn interrupt_line_valid() {
        let dev = PciBus
            .find_device_any(0x8086, &[0x100E, 0x100F, 0x10D3])
            .unwrap();
        let irq = (PciBus.read_config(&dev, 0x3C) & 0xFF) as u8;
        assert!(irq < 32, "IRQ line should be < 32, got {}", irq);
    }
}

pub use tests::suite;
