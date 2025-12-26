#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

pub mod qemu;
pub mod serial;
pub mod vga_buffer;

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

    loop {}
}

#[cfg(test)]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn _test_panic_handler(_info: &PanicInfo) -> ! {
    test_panic_handler(_info)
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
