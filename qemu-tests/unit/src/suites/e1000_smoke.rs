use crate::dma_alloc::BootDmaAllocator;
use crate::mmio_regs::MmioRegs;
use embclox_e1000::RegisterAccess;
use embclox_e1000::regs::*;

/// Global test context set by main before running suites.
static mut CTX: Option<E1000TestCtx> = None;

pub struct E1000TestCtx {
    pub regs: MmioRegs,
    pub kernel_offset: u64,
    pub phys_offset: u64,
}

/// Initialize the e1000 test context. Called once from main.
///
/// # Safety
/// Must be called before `suite()` and only from single-threaded init.
pub unsafe fn init(regs: MmioRegs, kernel_offset: u64, phys_offset: u64) {
    unsafe {
        *core::ptr::addr_of_mut!(CTX) = Some(E1000TestCtx {
            regs,
            kernel_offset,
            phys_offset,
        });
    }
}

fn ctx() -> &'static E1000TestCtx {
    unsafe {
        (*core::ptr::addr_of!(CTX))
            .as_ref()
            .expect("e1000 test context not initialized")
    }
}

#[embclox_test_macros::test_suite(name = "e1000_smoke")]
mod tests {
    use super::*;

    #[test]
    fn status_link_up() {
        let regs = &ctx().regs;
        let status = regs.read_reg(STAT);
        assert!(
            status & 0x2 != 0,
            "e1000 link should be up, STATUS={:#x}",
            status
        );
    }

    #[test]
    fn mac_address_nonzero() {
        let regs = &ctx().regs;
        let ral = regs.read_reg(RAL);
        let rah = regs.read_reg(RAH);
        assert!(ral != 0 || rah != 0, "MAC address should not be zero");
    }

    #[test]
    fn init_device_and_split() {
        let c = ctx();
        let dma = BootDmaAllocator {
            kernel_offset: c.kernel_offset,
            phys_offset: c.phys_offset,
        };
        let mut dev = embclox_e1000::E1000Device::new(c.regs, dma);
        let mac = dev.mac_address();
        assert!(mac != [0; 6], "MAC should not be all zeros");

        let (mut rx, mut tx) = dev.split();
        let received = rx.recv_with(|_data| {
            panic!("should not receive any packet");
        });
        assert!(received.is_none(), "no packets should be pending");

        let test_frame: [u8; 64] = [0xff; 64];
        tx.transmit(&test_frame);
    }
}

pub use tests::suite;
