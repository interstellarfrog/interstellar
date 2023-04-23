use uart_16550::SerialPort;
use spin::Mutex;
use lazy_static::lazy_static;
use core::{fmt::Write};
use x86_64::instructions::interrupts;

lazy_static! { // Only Init Once On First Use
    pub static ref SERIAL1: Mutex<SerialPort> = { // Wrap In SpinLock
        let mut serial_port = unsafe { SerialPort::new(0x3f8) }; // 0x3f8 Standard Port Addr For The First Serial Interface
        serial_port.init();
        Mutex::new(serial_port)
    };
}

#[doc(hidden)]
pub fn _print(args: ::core::fmt::Arguments) {
    interrupts::without_interrupts(|| { // Disable Interrupts
        SERIAL1.lock()
        .write_fmt(args)
        .expect("Serial Print Failed!");
    });
    
}

#[macro_export]
macro_rules! serial_print { // Prints To Main PC From OS Using Serial
    ($($arg:tt)*) => { $crate::serial::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! serial_println { // Adds Newline And Calls Serial_print
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}