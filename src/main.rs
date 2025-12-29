#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points
#![feature(custom_test_frameworks)]
#![test_runner(rust_os::test_runner)]
#![reexport_test_harness_main = "test_main"]
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;
use rust_os::{
    allocator, hlt_loop, init_kernel,
    memory::{self},
    println,
    task::{Task, executor::Executor, keyboard},
};
use x86_64::VirtAddr;

extern crate alloc;

#[cfg(not(test))]
use rust_os::print;

#[cfg(test)]
use rust_os::test_panic_handler;

entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello World{}", "!");

    init_kernel();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let mut executor = Executor::new();

    // Left as an example of calling an example task
    //executor.spawn(Task::new(number_task()));

    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();

    //let x = Box::new(4u64);
    //println!("heap value at {:p} is {}", x, *x);

    //let y = Box::new(41);
    //println!("heap value at {:p} is {}", y, *y);

    //let mut vec = Vec::new();
    //for i in 0..500 {
    //    vec.push(i);
    //}
    //println!("vec at {:p}", vec.as_slice());

    //let y = Box::new(41);
    //println!("heap value at {:p} is {}", y, *y);
    //// create a reference counted vector -> will be freed when count reaches 0
    //let reference_counted = Rc::new(vec![1, 2, 3]);
    //let cloned_reference = reference_counted.clone();
    //println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    //core::mem::drop(reference_counted);
    //println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    #[cfg(test)]
    test_main();

    hlt_loop()
}

// This and the function below it are left here as examples of
// implementing a task to run with the executor.
/*async fn async_number() -> u32 {
    42
}

async fn number_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}*/

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
