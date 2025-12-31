#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use bootloader_api::BootInfo;
use lazy_static::lazy_static;

use rust_os::{default_entry_point, gdt::{DOUBLE_FAULT_IST_INDEX, initialize_global_descriptor_table}, hlt_loop, qemu::{QemuExitCode, exit_qemu}, serial_print, serial_println};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

lazy_static! {
    // setup custom interrupt descriptor table, so we can exit qemu instead of panicing
    static ref TEST_INTERRUPT_DESCRIPTOR_TABLE: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault.set_handler_fn(test_double_fault_handler)
               .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

default_entry_point!(main);

fn main(_: &'static mut BootInfo) -> ! {
    serial_print!("double_fault::double_fault...\t");
    initialize_global_descriptor_table();
    TEST_INTERRUPT_DESCRIPTOR_TABLE.load();

    stack_overflow();

    // should actually run into a double fault before this
    panic!("Execution continued after stack overflow")
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read();
}

extern "x86-interrupt" fn test_double_fault_handler(_: InterruptStackFrame, _: u64) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    hlt_loop()
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    rust_os::test_panic_handler(_info)
}
