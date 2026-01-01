#![no_std]
#![no_main]

use core::panic::PanicInfo;
use bootloader_api::BootInfo;
use rust_os::{default_entry_point, qemu::{QemuExitCode, exit_qemu}, serial_print, serial_println};

default_entry_point!(main);

fn main(_: &'static mut BootInfo) -> ! {
    should_fail();
    serial_println!("[test did not panic]");
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

fn should_fail() {
    serial_print!("should_panic::should_fail...\t");
    assert_eq!(0, 1);
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
