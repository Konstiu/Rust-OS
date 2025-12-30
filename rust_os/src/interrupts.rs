use lazy_static::lazy_static;
use pc_keyboard::{DecodedKey, Keyboard, ScancodeSet1, layouts};
use spin::Mutex;
use x86_64::{instructions::port::Port, structures::idt::{PageFaultErrorCode, InterruptDescriptorTable, InterruptStackFrame}};
use pic8259::ChainedPics;
use x86_64::registers::control::Cr2;

use crate::{gdt, print, println, hlt_loop};
use crate::framebuffer::{framebuffer_size, put_pixel, Rgb, draw_cell, clear_color, reset_cursor};
use crate::wasm_game;

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode,) {
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

lazy_static! {
    static ref INTERRUPT_DESCRIPTOR_TABLE: InterruptDescriptorTable = {
        let mut interrupt_descriptor_table = InterruptDescriptorTable::new();
        interrupt_descriptor_table.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            interrupt_descriptor_table.double_fault.set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        interrupt_descriptor_table[u8::from(InterruptIndex::Timer)].set_handler_fn(timer_interrupt_handler);
        interrupt_descriptor_table[u8::from(InterruptIndex::Keyboard)].set_handler_fn(keyboard_interrupt_handler);
        interrupt_descriptor_table.page_fault.set_handler_fn(page_fault_handler);
        interrupt_descriptor_table
    };
}

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
        Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            pc_keyboard::HandleControl::Ignore
        ));
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
    Keyboard
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
    //print!(".");

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(u8::from(InterruptIndex::Timer));
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_: InterruptStackFrame) {
    let mut keyboard = KEYBOARD.lock();
    let mut ps2_port: Port<u8> =  Port::new(0x60);
    let scancode = unsafe { ps2_port.read() };

    if let Ok(Some(key_event)) = keyboard.add_byte(scancode)
        && let Some(key) = keyboard.process_keyevent(key_event) {
            reset_cursor(); 
            match key {
                DecodedKey::Unicode(character) => print!("{character}"),
                DecodedKey::RawKey(raw_key) => print!("{raw_key:?}")
            };
            let key_code: u8 = match key {
                DecodedKey::Unicode(c) => c as u8,
                DecodedKey::RawKey(code) => code as u8,
            };
            wasm_game::handle_key(key_code);
            wasm_game::update_game();  // Add this
            wasm_game::render_game(); 
            //wasm_game::handle_key(key_code);
        }
    
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(u8::from(InterruptIndex::Keyboard));
    }
}

#[cfg(test)]
mod tests {
    #[test_case]
    fn test_breakpoint_interrupt() {
        x86_64::instructions::interrupts::int3();
    }   
}
