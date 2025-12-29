#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
use core::panic::PanicInfo;
use rust_os::{allocator, default_entry_point, hlt_loop, init_kernel, memory::{self, BootInfoFrameAllocator}, println};
use bootloader_api::{BootInfo};
use x86_64::VirtAddr;

extern crate alloc;

#[cfg(not(test))]
use rust_os::print;

#[cfg(test)]
use rust_os::test_panic_handler;

default_entry_point!(kernel_main);
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    println!("Hello World{}", "!");

    init_kernel();

    let phys_mem_offset = VirtAddr::new(
        boot_info.physical_memory_offset.into_option().expect("Could not obtain physical memory offset from bootloader")
    );
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {BootInfoFrameAllocator::init(&boot_info.memory_regions)};

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
    test_main();

    hlt_loop()
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print!("{}", _info);
    hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

