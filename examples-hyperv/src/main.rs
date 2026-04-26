#![no_std]
#![no_main]

extern crate alloc;
extern crate embclox_hal_x86;

mod framebuffer;

use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping, entry_point};
use core::panic::PanicInfo;

const BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    // Minimal test: write directly to framebuffer WITHOUT HAL init.
    // This isolates whether the framebuffer works on Hyper-V Gen2.
    if let Some(fb) = boot_info.framebuffer.as_mut() {
        let info = fb.info();
        let buf = fb.buffer_mut();

        // Fill entire framebuffer with white
        for byte in buf.iter_mut() {
            *byte = 0xFF;
        }

        // Draw a red rectangle in the top-left
        let bpp = info.bytes_per_pixel;
        let stride = info.stride;
        for y in 10..60 {
            for x in 10..200 {
                let offset = (y * stride + x) * bpp;
                if offset + bpp <= buf.len() {
                    buf[offset] = 0x00; // B
                    buf[offset + 1] = 0x00; // G
                    buf[offset + 2] = 0xFF; // R
                    if bpp > 3 {
                        buf[offset + 3] = 0xFF; // A
                    }
                }
            }
        }
    }

    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
