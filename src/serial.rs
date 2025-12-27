use core::fmt::Write;
use core::fmt::Arguments;

use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;
use x86_64::instructions::interrupts::without_interrupts;


lazy_static! {
    static ref SERIAL_PORT: Mutex<SerialPort> = {
        let mut serial_port = unsafe {
            SerialPort::new(0x3F8)
        };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

// Prints to qemu host through uart 16550 serial port
// appends a newline
#[macro_export]
macro_rules! serial_println {
    () => {
        $crate::serial_print!("\n")
    };
    ($fmt:expr) => {
        $crate::serial_print!(concat!($fmt, "\n"))
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::serial_print!(concat!($fmt, "\n"), $($arg)*)
    };
}

// Prints to qemu host through uart 16550 serial port
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_serial_print(format_args!($($arg)*));
    };
}

#[doc(hidden)]
pub fn _serial_print(args: Arguments) {
    without_interrupts(|| {
     SERIAL_PORT.lock().write_fmt(args).expect("Failed to print to serial port");       
    })
}
