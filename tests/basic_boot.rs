#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use rust_os::{hlt_loop, println};

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    test_main();
    hlt_loop()
}

#[panic_handler]
fn test_panic_handler(_info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(_info)
}

#[test_case]
fn trivial_integration_test() {
    println!("Tests _start entry point and simple print statement")
}
