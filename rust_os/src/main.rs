#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
use bootloader_api::BootInfo;
use core::panic::PanicInfo;
use core::slice;
use rust_os::task::executor::Executor;
use rust_os::task::{Task, shell};
use rust_os::{default_entry_point, hlt_loop, init_kernel};

extern crate alloc;

#[allow(dead_code)]
static SNAKE_WASM: &[u8] = include_bytes!("wasm/snake.wasm");
#[allow(dead_code)]
static COWSAY_WASM: &[u8] = include_bytes!("wasm/cowsay.wasm");

#[cfg(test)]
use rust_os::test_panic_handler;

default_entry_point!(kernel_main);
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let ramdisk_addr = boot_info
        .ramdisk_addr
        .into_option()
        .expect("Could not get ramdisk address from boot info");

    let ramdisk =
        unsafe { slice::from_raw_parts(ramdisk_addr as *const u8, boot_info.ramdisk_len as usize) };
    init_kernel(boot_info);

    #[cfg(not(test))]
    {
        rust_os::filesystem::init_filesystem(ramdisk).expect("Failed to initialize filesystem");
        let mut executor = Executor::new();
        executor.spawn(Task::new(shell::run()));
        executor.run();
    }

    #[cfg(test)]
    test_main();

    #[allow(unreachable_code)]
    hlt_loop()
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    use rust_os::serial_print;
    serial_print!("{}", _info);
    hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
