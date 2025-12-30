#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
use core::panic::PanicInfo;
use rust_os::{allocator, default_entry_point, hlt_loop, init_kernel, memory::{self, BootInfoFrameAllocator}, println};
use bootloader_api::{BootInfo};
use x86_64::VirtAddr;
use rust_os::framebuffer::{framebuffer_size, put_pixel, Rgb, draw_cell, clear_color};
use rust_os::wasm_game;

extern crate alloc;

static SNAKE_WASM: &[u8] = include_bytes!("wasm/snake.wasm");

#[cfg(not(test))]
use rust_os::print;

#[cfg(test)]
use rust_os::test_panic_handler;

default_entry_point!(kernel_main);
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {

    init_kernel(boot_info.framebuffer.as_mut().expect("Could not get framebuffer from boot info"));
    
    //println!("Hello World{}", "!");

    let phys_mem_offset = VirtAddr::new(
        boot_info.physical_memory_offset.into_option().expect("Could not obtain physical memory offset from bootloader")
    );
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {BootInfoFrameAllocator::init(&boot_info.memory_regions)};

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");



    //println!("Initializing WASM game...");
    wasm_game::init_wasm_game(SNAKE_WASM);
   // println!("WASM game initialized!");
    wasm_game::render_game();




    #[cfg(test)]
    test_main();

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

