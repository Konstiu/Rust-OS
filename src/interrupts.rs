use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use pic8259::ChainedPics;

use crate::{gdt, print, println};


lazy_static! {
    static ref INTERRUPT_DESCRIPTOR_TABLE: InterruptDescriptorTable = {
        let mut interrupt_descriptor_table = InterruptDescriptorTable::new();
        interrupt_descriptor_table.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            interrupt_descriptor_table.double_fault.set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        interrupt_descriptor_table[u8::from(InterruptIndex::Timer)].set_handler_fn(timer_interrupt_handler);
        interrupt_descriptor_table
    };
}

const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

static PICS: spin::Mutex<ChainedPics> = Mutex::new(unsafe {
    ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)
});

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum InterruptIndex {
    Timer = PIC_1_OFFSET,
}

impl From<InterruptIndex> for u8 {
    fn from(value: InterruptIndex) -> Self {
        value as u8
    }
}

pub fn initialize_interrupt_handling() {
    INTERRUPT_DESCRIPTOR_TABLE.load();
    unsafe { PICS.lock().initialize(); }
    x86_64::instructions::interrupts::enable();
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

extern "x86-interrupt" fn timer_interrupt_handler(_: InterruptStackFrame) {
    print!(".");

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(u8::from(InterruptIndex::Timer));
    }
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn test_breakpoint_interrupt() {
        x86_64::instructions::interrupts::int3();
    }   
}
