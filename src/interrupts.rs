use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::{gdt, println};

lazy_static! {
    static ref INTERRUPT_DESCRIPTOR_TABLE: InterruptDescriptorTable = {
        let mut interrupt_descriptor_table = InterruptDescriptorTable::new();
        interrupt_descriptor_table.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            interrupt_descriptor_table.double_fault.set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        interrupt_descriptor_table
    };
}

pub fn load_interrupt_descriptor_table() {
    INTERRUPT_DESCRIPTOR_TABLE.load();
}

extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT HIT\n{:#?}", frame)
}

// occurs on a specific combination of exceptions.
// e.g., an interrupt for which no handler is set up may lead to a double fault
// an error such as a page fault during an interrupt handler may lead to a double fault
// ...
extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _: u64) -> ! {
    // we can't actually continue after a double fault
    panic!("EXCEPTION: DOUBLE FAULT OCCURED\n{:#?}", frame)
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn test_breakpoint_interrupt() {
        x86_64::instructions::interrupts::int3();
    }   
}
