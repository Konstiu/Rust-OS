#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use x86_64::instructions::hlt;
use bootloader_api::{BootInfo, info::FrameBuffer};

pub mod gdt;
pub mod interrupts;
pub mod qemu;
pub mod serial;
pub mod framebuffer;
pub mod memory;
pub mod allocator;
pub mod entry_point;

extern crate alloc;

pub fn init_kernel(framebuffer: &'static mut FrameBuffer) {
    
    gdt::initialize_global_descriptor_table();
    framebuffer::init_framebuffer_writer(framebuffer);
    interrupts::initialize_interrupt_handling();
}

pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    use crate::qemu::exit_qemu;

    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }

    exit_qemu(qemu::QemuExitCode::Success);
}

pub fn test_panic_handler(_info: &PanicInfo) -> ! {
    use crate::qemu::exit_qemu;

    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", _info);
    exit_qemu(qemu::QemuExitCode::Failed);

    hlt_loop()
}

pub fn hlt_loop() -> ! {
    loop {
        hlt();
    }
}

#[cfg(test)]
default_entry_point!(test_kernel_main);

#[cfg(test)]
fn test_kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    init_kernel();
    test_main();
    hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn _test_panic_handler(_info: &PanicInfo) -> ! {
    test_panic_handler(_info)
}

#[test_case]
#[allow(clippy::eq_op)]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
