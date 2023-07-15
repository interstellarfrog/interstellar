//Copyright (C) <2023>  <interstellarfrog>
//
//This program is free software: you can redistribute it and/or modify
//it under the terms of the GNU General Public License as published by
//the Free Software Foundation, either version 3 of the License, or
//(at your option) any later version.
//
//This program is distributed in the hope that it will be useful,
//but WITHOUT ANY WARRANTY; without even the implied warranty of
//MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//GNU General Public License for more details.
//
//You should have received a copy of the GNU General Public License
//along with this program.  If not, see <https://www.gnu.org/licenses/>.

// This module is for writing to serial which is written through QEMU to stdout

use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;
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
    interrupts::without_interrupts(|| {
        // Disable Interrupts
        SERIAL1
            .lock()
            .write_fmt(args)
            .expect("Serial Print Failed!");
    });
}

#[macro_export]
macro_rules! serial_print { // Prints To Main PC From Kernel Using Serial
    ($($arg:tt)*) => { $crate::drivers::io::serial::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! serial_println { // Adds Newline And Calls Serial_print
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}
