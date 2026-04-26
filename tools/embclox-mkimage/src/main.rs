use std::path::PathBuf;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args.len() > 4 {
        eprintln!("Usage: {} <kernel-elf> <output.img> [--bios]", args[0]);
        std::process::exit(1);
    }
    let kernel = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);
    let bios = args.get(3).is_some_and(|a| a == "--bios");

    if !kernel.exists() {
        eprintln!("Kernel ELF not found: {}", kernel.display());
        std::process::exit(1);
    }

    let builder = bootloader::DiskImageBuilder::new(kernel);

    if bios {
        builder
            .create_bios_image(&output)
            .expect("Failed to create BIOS disk image");
        println!("Created BIOS image: {}", output.display());
    } else {
        builder
            .create_uefi_image(&output)
            .expect("Failed to create UEFI disk image");
        println!("Created UEFI image: {}", output.display());
    }
}
