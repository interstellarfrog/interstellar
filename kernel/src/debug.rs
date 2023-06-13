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

#[cfg(debug_assertions)]
use spin::Mutex;

#[cfg(debug_assertions)]
/// Global flag indicating whether debug mode is enabled or not.
pub static mut DEBUG_MODE: bool = false;

#[cfg(debug_assertions)]
/// Sets the debug mode.
///
/// # Arguments
///
/// * `enable` - A boolean value indicating whether to enable or disable the debug mode.
pub fn set_debug_mode(enable: bool) {
    unsafe {
        DEBUG_MODE = enable;
    }
}

#[cfg(debug_assertions)]
lazy_static::lazy_static! {
    /// A mutex used for synchronization of debug printing.
    pub static ref DEBUG_LOCK: Mutex<()> = Mutex::new(());
}

#[cfg(debug_assertions)]
#[macro_export]
/// Macro for printing debug messages with file and line information.
macro_rules! real_debug_println {
    ($($arg:tt)*) => {
        if unsafe { DEBUG_MODE } {
            let _lock = DEBUG_LOCK.lock();
            let line = line!();
            let file = file!();
            println!("DEBUG: [{}:{}] {}", file, line, format_args!($($arg)*));
            serial_println!("DEBUG: [{}:{}] {}", file, line, format_args!($($arg)*));
        }
    };
}

#[macro_export]
/// Macro for printing debug info
/// 
/// The debug info will not be printed if building in release mode
/// 
/// And the rust compiler should optimize it away
macro_rules! debug_println {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            real_debug_println!($($arg)*);
        }
    };
}
