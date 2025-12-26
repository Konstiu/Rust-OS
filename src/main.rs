#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

#[cfg(test)]
use rust_os::test_panic_handler;

mod qemu;
mod serial;
mod vga_buffer;
mod interrupts;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    // allow CPU to handle interrupts
    interrupts::load_interrupt_descriptor_table();

    #[cfg(test)]
    test_main();

    loop {}
}

fn get_bad_index() -> usize {
    10 // Compiler might not optimize this in debug mode
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print!("{}", _info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}


