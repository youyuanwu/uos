#![no_std]
#![no_main]

extern crate alloc;
extern crate embclox_hal_x86;

mod harness;
mod suites;

use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping, entry_point};
use core::panic::PanicInfo;
use embclox_core::mmio_regs::MmioRegs;
use embclox_e1000::RegisterAccess;
use embclox_e1000::regs::*;
use log::*;

const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let mut p = embclox_hal_x86::init(boot_info, embclox_hal_x86::Config::default());
    info!("=== embclox test runner ===");

    let mut total = 0usize;

    // --- HAL tests (no device setup needed) ---

    // hal_pci: PciBus is zero-sized, no init needed
    let (name, tests) = suites::hal_pci::suite();
    total += harness::run_suite(name, tests);

    // hal_memory: init with a MemoryMapper, run before e1000 maps BAR0
    // (tests map/unmap cleanly, leaving pages free for later use)
    unsafe {
        suites::hal_memory::init(p.memory.phys_offset(), p.memory.kernel_offset());
    }
    let (name, tests) = suites::hal_memory::suite();
    total += harness::run_suite(name, tests);

    // --- e1000 tests (need PCI scan, BAR0 map, device reset) ---

    let pci_dev = p
        .pci
        .find_device_any(0x8086, &[0x100E, 0x100F, 0x10D3])
        .expect("e1000 device not found on PCI bus");
    let bar0_phys = p.pci.read_bar(&pci_dev, 0);
    let e1000_mmio = p.memory.map_mmio(bar0_phys, 0x20000);
    let regs = MmioRegs::new(e1000_mmio.vaddr());
    e1000_reset(&regs);
    p.pci.enable_bus_mastering(&pci_dev);

    unsafe {
        suites::e1000_smoke::init(
            regs,
            embclox_core::dma_alloc::BootDmaAllocator {
                kernel_offset: p.memory.kernel_offset(),
                phys_offset: p.memory.phys_offset(),
            },
        );
    }
    let (name, tests) = suites::e1000_smoke::suite();
    total += harness::run_suite(name, tests);

    // Clean up MMIO mapping
    // Safety: e1000_smoke tests are done, no references to mapped memory remain.
    unsafe { p.memory.unmap_mmio(&e1000_mmio) };

    info!("=== {} passed ===", total);
    harness::qemu_exit(0);
}

fn e1000_reset(regs: &MmioRegs) {
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
    info!("e1000 device reset complete");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("PANIC: {}", info);
    harness::qemu_exit(1);
}
