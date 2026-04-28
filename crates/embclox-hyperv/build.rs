use std::env;
use std::path::PathBuf;

fn main() {
    let include_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("include");

    // Re-run if any header changes
    println!("cargo:rerun-if-changed={}", include_dir.display());

    // Generate bindings for NVSP/RNDIS wire-format structs
    let bindings = bindgen::Builder::default()
        .header(include_dir.join("hyperv_net_bindgen.h").to_str().unwrap())
        .header(include_dir.join("rndis.h").to_str().unwrap())
        .header(include_dir.join("hyperv_vmbus.h").to_str().unwrap())
        // Freestanding mode: no libc headers needed when cross-compiling to bare metal
        .clang_args(["-ffreestanding", "-nostdinc"])
        .use_core()
        .derive_debug(true)
        .derive_default(true)
        .derive_copy(true)
        .prepend_enum_name(false)
        .generate()
        .expect("failed to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("hyperv_bindings.rs"))
        .expect("failed to write bindings");
}
