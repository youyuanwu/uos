use embclox_core::dma_alloc::BootDmaAllocator;
use embclox_core::e1000_embassy::E1000Embassy;
use embclox_core::mmio_regs::MmioRegs;
use embclox_e1000::RegisterAccess;
use embclox_e1000::regs::*;

use embassy_net::{Ipv4Address, Ipv4Cidr, StackResources, StaticConfigV4};
use embassy_net_driver::Driver;
use static_cell::StaticCell;

/// Global test context.
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
            .expect("embassy test context not initialized")
            .0
    }
}

fn dma() -> &'static BootDmaAllocator {
    unsafe {
        &(*core::ptr::addr_of!(CTX))
            .as_ref()
            .expect("embassy test context not initialized")
            .1
    }
}

/// Helper: reset and create a fresh e1000 device.
fn new_device() -> embclox_e1000::E1000Device<MmioRegs, BootDmaAllocator> {
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

/// Embassy adapter and networking stack tests.
#[embclox_test_macros::test_suite(name = "e1000_embassy")]
mod tests {
    use super::*;

    /// Verify E1000Embassy adapter wraps device correctly and
    /// reports the right MAC and capabilities.
    #[test]
    fn embassy_adapter_capabilities() {
        let dev = new_device();
        let mac = dev.mac_address();
        let adapter = E1000Embassy::new(dev, mac);

        let caps = adapter.capabilities();
        assert_eq!(caps.max_transmission_unit, 1514, "MTU should be 1514");

        let hw_addr = adapter.hardware_address();
        match hw_addr {
            embassy_net_driver::HardwareAddress::Ethernet(addr) => {
                assert_eq!(addr, mac, "hardware_address should match MAC");
            }
            _ => panic!("expected Ethernet hardware address"),
        }
    }

    /// Create a full Embassy network stack with static IP.
    /// Verify the stack initializes without panic.
    #[test]
    fn embassy_stack_init() {
        let dev = new_device();
        let mac = dev.mac_address();
        let adapter = E1000Embassy::new(dev, mac);

        let config = embassy_net::Config::ipv4_static(StaticConfigV4 {
            address: Ipv4Cidr::new(Ipv4Address::new(10, 0, 2, 15), 24),
            gateway: Some(Ipv4Address::new(10, 0, 2, 2)),
            dns_servers: Default::default(),
        });

        static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
        let resources = RESOURCES.init(StackResources::new());
        let seed = 0x1234_5678u64;

        let (_stack, _runner) = embassy_net::new(adapter, config, resources, seed);
        log::info!("Embassy stack initialized successfully");
    }
}

pub use tests::suite;
