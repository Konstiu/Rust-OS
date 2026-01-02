use std::{
    path::{Path, PathBuf},
    process::ExitStatus,
};

use clap::Parser;
use qemu_runner::{QemuMode, run_qemu_with_kernel};

// QEMU with isa-debug-exit returns (port << 1) | 1.
const SUCCESS_EXIT_CODE: i32 = 33;


#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    ramdisk: Option<PathBuf>,

    #[arg(value_name = "KERNEL")]
    kernel: PathBuf
}

fn main() {
    let args = Args::parse();
    
    let kernel_path = args.kernel;
    let ramdisk_path = args.ramdisk;
    let is_test = is_test_binary(&kernel_path);

    let mode = if is_test {
        QemuMode::Test
    } else {
        QemuMode::Run
    };

    let status = match run_qemu_with_kernel(kernel_path, ramdisk_path, mode) {
        Ok(status) => status,
        Err(err) => {
            eprintln!("failed to run qemu: {err}");
            std::process::exit(1);
        }
    };

    exit_with_status(status);
}

fn exit_with_status(status: ExitStatus) -> ! {
    match status.code() {
        Some(code) if code == SUCCESS_EXIT_CODE => std::process::exit(0),
        Some(code) => {
            eprintln!("qemu exited with status {code}");
            std::process::exit(1);
        }
        None => {
            eprintln!("qemu terminated by signal");
            std::process::exit(1);
        }
    }
}

fn is_test_binary(path: &Path) -> bool {
    let parent = match path.parent() {
        Some(parent) => parent,
        None => return false,
    };
    let dir_name = match parent.file_name().and_then(|name| name.to_str()) {
        Some(name) => name,
        None => return false,
    };
    dir_name == "deps" || dir_name.starts_with("rustdoctest")
}
