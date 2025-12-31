use std::path::PathBuf;

use qemu_runner::{QemuMode, run_qemu_with_image};

fn main() {
    let bios_path = PathBuf::from(env!("BIOS_PATH"));
    let status = match run_qemu_with_image(&bios_path, QemuMode::Run) {
        Ok(status) => status,
        Err(err) => {
            eprintln!("failed to run qemu: {err}");
            std::process::exit(1);
        }
    };

    std::process::exit(status.code().unwrap_or(1));
}
