#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
use bootloader_api::BootInfo;
use core::{panic::PanicInfo, ptr::slice_from_raw_parts};
use rust_os::{
    allocator, default_entry_point, filesystem, hlt_loop, init_kernel, memory::{self, BootInfoFrameAllocator}, println
};
use x86_64::VirtAddr;
use rust_os::wasm_game;

extern crate alloc;

#[allow(dead_code)]
static SNAKE_WASM: &[u8] = include_bytes!("wasm/snake.wasm");
#[allow(dead_code)]
static COWSAY_WASM: &[u8] = include_bytes!("wasm/cowsay.wasm");

#[cfg(not(test))]
use rust_os::task::{Task, executor::Executor, keyboard};

#[cfg(test)]
use rust_os::test_panic_handler;

default_entry_point!(kernel_main);
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    init_kernel(
        boot_info
            .framebuffer
            .as_mut()
            .expect("Could not get framebuffer from boot info"),
    );


    let phys_mem_offset = VirtAddr::new(
        boot_info
            .physical_memory_offset
            .into_option()
            .expect("Could not obtain physical memory offset from bootloader"),
    );
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let ramdisk: &[u8] = unsafe {
        &*slice_from_raw_parts(
            boot_info.ramdisk_addr.into_option().expect("Could not get ramdisk address. Ramdisk may not be set") as *const u8,
            boot_info.ramdisk_len.try_into().expect("Could not convert ramdisk length to usize")
        )
    };

    let mut tarfs = filesystem::create_tarfs(ramdisk);
    for entry in tarfs.list() {
        let entity = entry.expect("Could not get entity");
        println!("Entry: {}", entity.name)
    }


    //wasm_game::init_wasm_game(SNAKE_WASM);
    //wasm_game::render_game();

    wasm_game::init_wasm_game(COWSAY_WASM);
    wasm_game::render_game();


    #[cfg(not(test))]
        {
            let mut executor = Executor::new();
            executor.spawn(Task::new(keyboard::print_keypresses()));
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
