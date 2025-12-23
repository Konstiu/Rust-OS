#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;
mod vga_buffer;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");

    let x = [1, 2, 3];
    println!("Value: {}", x[get_bad_index()]);

    loop {}
}

fn get_bad_index() -> usize {
    10 // Compiler might not optimize this in debug mode
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    print!("{}", _info);
    loop {}
}
