use embclox_core::dma_alloc::BootDmaAllocator;
use embclox_core::mmio_regs::MmioRegs;
use embclox_e1000::RegisterAccess;
use embclox_e1000::regs::*;

/// Global test context — same pattern as e1000_smoke.
static mut CTX: Option<(MmioRegs, BootDmaAllocator)> = None;

/// # Safety
/// Must be called before `suite()` and only from single-threaded init.
pub unsafe fn init(regs: MmioRegs, dma: BootDmaAllocator) {
    unsafe {
        *core::ptr::addr_of_mut!(CTX) = Some((regs, dma));
    }
}

fn regs() -> &'static MmioRegs {
    unsafe {
        &(*core::ptr::addr_of!(CTX))
            .as_ref()
            .expect("e1000 driver test context not initialized")
            .0
    }
}

fn dma() -> &'static BootDmaAllocator {
    unsafe {
        &(*core::ptr::addr_of!(CTX))
            .as_ref()
            .expect("e1000 driver test context not initialized")
            .1
    }
}

/// Helper: create a fresh e1000 device (reset + init).
/// Device is reset on drop, so each test gets clean state.
fn new_device() -> embclox_e1000::E1000Device<MmioRegs, BootDmaAllocator> {
    // Reset before init (required by E1000Device::new contract)
    let r = regs();
    r.write_reg(IMS, 0);
    let ctl = r.read_reg(CTL);
    r.write_reg(CTL, ctl | CTL_RST);
    let mut timeout = 100_000u32;
    while r.read_reg(CTL) & CTL_RST != 0 {
        timeout -= 1;
        assert!(timeout > 0, "e1000 reset timeout");
    }
    r.write_reg(IMS, 0);
    r.write_reg(CTL, CTL_SLU | CTL_ASDE);
    r.write_reg(FCAL, 0);
    r.write_reg(FCAH, 0);
    r.write_reg(FCT, 0);
    r.write_reg(FCTTV, 0);

    embclox_e1000::E1000Device::new(*r, dma().clone())
}

/// E1000 driver tests — each test creates and drops its own device,
/// getting a full reset cycle between tests.
#[embclox_test_macros::test_suite(name = "e1000_driver")]
mod tests {
    use super::*;

    /// Verify link_is_up() returns true on QEMU.
    #[test]
    fn link_is_up() {
        let dev = new_device();
        assert!(dev.link_is_up(), "link should be up on QEMU e1000");
    }

    /// Verify MAC address is consistent across device reinit.
    #[test]
    fn mac_consistent_across_reinit() {
        let dev1 = new_device();
        let mac1 = dev1.mac_address();
        drop(dev1);

        let dev2 = new_device();
        let mac2 = dev2.mac_address();
        assert_eq!(mac1, mac2, "MAC should be the same after device reset");
    }

    /// Transmit multiple frames and verify TX doesn't stall.
    #[test]
    fn transmit_multiple_frames() {
        let mut dev = new_device();
        let (_, mut tx) = dev.split();

        for i in 0u8..10 {
            let mut frame = [0u8; 64];
            frame[0..6].fill(0xff); // broadcast dest
            frame[12] = i; // vary payload
            tx.transmit(&frame);
        }
    }

    /// Transmit a maximum-sized Ethernet frame (1514 bytes).
    #[test]
    fn transmit_max_frame() {
        let mut dev = new_device();
        let (_, mut tx) = dev.split();

        let mut frame = [0u8; 1514];
        frame[0..6].fill(0xff);
        tx.transmit(&frame);
    }

    /// Enable and disable interrupts, verify IMS register reflects state.
    #[test]
    fn interrupt_enable_disable() {
        let dev = new_device();

        dev.enable_interrupts();
        let ims = regs().read_reg(IMS);
        assert!(ims != 0, "IMS should be non-zero after enable");

        dev.disable_interrupts();
        let ims = regs().read_reg(IMS);
        assert_eq!(ims, 0, "IMS should be zero after disable");
    }

    /// Verify transmit_with zero-copy API works.
    #[test]
    fn transmit_with_zero_copy() {
        let mut dev = new_device();
        let (_, mut tx) = dev.split();

        let result = tx.transmit_with(64, |buf| {
            buf[0..6].fill(0xff); // broadcast
            buf[6..12].fill(0xaa); // src
            buf[12] = 0x08;
            buf[13] = 0x00; // IPv4 ethertype
            42 // return value
        });
        assert_eq!(
            result.unwrap(),
            42,
            "transmit_with should return closure result"
        );
    }
}

pub use tests::suite;
