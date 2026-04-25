use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <kernel-elf> <output.img>", args[0]);
        std::process::exit(1);
    }
    let kernel = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);

    if !kernel.exists() {
        eprintln!("Kernel ELF not found: {}", kernel.display());
        std::process::exit(1);
    }

    bootloader::DiskImageBuilder::new(kernel)
        .create_bios_image(&output)
        .expect("Failed to create BIOS disk image");

    println!("Created BIOS image: {}", output.display());
}
