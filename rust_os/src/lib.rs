#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader_api::BootInfo;
use core::panic::PanicInfo;
use x86_64::{VirtAddr, instructions::hlt};

pub mod allocator;
pub mod entry_point;
pub mod filesystem;
pub mod framebuffer;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod qemu;
pub mod serial;
pub mod task;
pub mod wasm_game;

extern crate alloc;

pub fn init_kernel(boot_info: &'static mut BootInfo) {
    gdt::initialize_global_descriptor_table();
    let framebuffer = boot_info
        .framebuffer
        .as_mut()
        .expect("Could not get framebuffer from boot info");
    framebuffer::init_framebuffer_writer(framebuffer);
    interrupts::initialize_interrupt_handling();

    let phys_mem_offset = VirtAddr::new(
        boot_info
            .physical_memory_offset
            .into_option()
            .expect("Could not obtain physical memory offset from bootloader"),
    );
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
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
fn test_kernel_main(boot_info: &'static mut BootInfo) -> ! {
    init_kernel(boot_info);
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
