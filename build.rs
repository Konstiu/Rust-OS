use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").expect("Failed to get out_dir"));
    let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_RUST_OS_rust_os").expect("Failed to get kernel"));

    let bios_path = out_dir.join("bios.img");
    bootloader::BiosBoot::new(&kernel)
        .create_disk_image(&bios_path)
        .expect("Failed to create disk image");
    
    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());
}
