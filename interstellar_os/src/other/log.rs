use crate::println;
use crate::serial_println;
use conquer_once::spin::OnceCell;
use spinning_top::Spinlock;

/// Max backtrace entries in the [BACKTRACE] list
pub const MAX_BACKTRACE_ENTRIES: usize = 7;
/// List of backtrace entries
pub static BACKTRACE: OnceCell<Spinlock<[Option<BacktraceEntry>; MAX_BACKTRACE_ENTRIES]>> =
    OnceCell::uninit();
/// Index of the next backtrace
pub static BACKTRACE_INDEX: OnceCell<Spinlock<usize>> = OnceCell::uninit();

/// [BACKTRACE] entry struct for tracing
#[derive(Clone, Copy)]
pub struct BacktraceEntry {
    pub message: [u8; 64], // Change the message field to a fixed-size array
    pub file: &'static str,
    pub line: u32,
}

impl BacktraceEntry {
    /// Creates a new backtrace entry
    pub fn new(message: Option<&str>, file: &'static str, line: u32) -> BacktraceEntry {
        let mut msg_buffer = [0u8; 64];
        if let Some(message) = message {
            let bytes = message.as_bytes();
            let len = bytes.len().min(63);
            msg_buffer[..len].copy_from_slice(&bytes[..len]);
            if len < 63 {
                msg_buffer[len..].fill(b' '); // Fill remaining bytes with spaces
            }
        }
        BacktraceEntry {
            message: msg_buffer,
            file,
            line,
        }
    }
}

/// enum of different log levels
#[derive(PartialEq, PartialOrd)]
pub enum LogLevel {
    /// Does not log
    None,
    /// Only logs errors
    Low,
    /// Normal level logs warnings, errors, etc.
    Normal,
    /// Logs info about initialization entering/exiting important functions
    High,
    /// Highest level of logging
    Beyond,
}

/// A [Logger] that the whole kernel can use once initialized
pub static LOGGER: OnceCell<Spinlock<Logger>> = OnceCell::uninit();

/// A Logger for the kernel
///
/// # Fields
/// * `screen_printing` - [bool] - To print to the framebuffer or not
///
/// * `serial_printing` - [bool] - To print to the serial interface or not
///
/// * `tracing` - [bool] - To enable tracing or not
///
/// * `log_level` - [LogLevel] - The Highest level to log
pub struct Logger {
    /// To print to the framebuffer or not
    pub screen_printing: bool,
    /// To print to the serial interface or not
    pub serial_printing: bool,
    /// To enable tracing or not
    pub tracing: bool,
    /// The Highest level to log
    pub log_level: LogLevel,
}

impl Logger {
    /// Changes the log level
    pub fn set_log_level(&mut self, log_level: LogLevel) {
        self.log_level = log_level;
    }

    /// Logs an error
    ///
    /// Depending on the [Logger]s [LogLevel] and how the [Logger] is set up this may print to the screen or the serial
    ///
    /// This logs if the log level is Low or more
    pub fn error(&self, message: &str) {
        if self.log_level >= LogLevel::Low {
            if self.serial_printing {
                serial_println!("{}", message);
            }
            if self.screen_printing {
                println!("{}", message);
            }
        }
    }

    /// Logs a warning
    ///
    /// Depending on the [Logger]s [LogLevel] and how the [Logger] is set up this may print to the screen or the serial
    ///
    /// This logs if the log level is Normal or more
    pub fn warn(&self, message: &str) {
        if self.log_level >= LogLevel::Normal {
            if self.serial_printing {
                serial_println!("{}", message);
            }
            if self.screen_printing {
                println!("{}", message);
            }
        }
    }

    /// Logs info
    ///
    /// Depending on the [Logger]s [LogLevel] and how the [Logger] is set up this may print to the screen or the serial
    ///
    /// This logs if the log level is High or more
    pub fn info(&self, message: &str) {
        if self.log_level >= LogLevel::High {
            if self.serial_printing {
                serial_println!("{}", message);
            }
            if self.screen_printing {
                println!("{}", message);
            }
        }
    }

    /// Logs a debug message
    ///
    /// Depending on the [Logger]s [LogLevel] and how the [Logger] is set up this may print to the screen or the serial
    ///
    /// This logs if the log level is Beyond or more
    pub fn debug(&self, message: &str) {
        if self.log_level == LogLevel::Beyond {
            if self.serial_printing {
                serial_println!("{}", message);
            }
            if self.screen_printing {
                println!("{}", message);
            }
        }
    }

    /// Used to add a trace to the [BACKTRACE] list
    ///
    /// The list is managed automatically the list size can be set through [MAX_BACKTRACE_ENTRIES]
    pub fn trace(&self, message: Option<&str>, file: &'static str, line: u32) {
        if self.tracing {
            let bte = BacktraceEntry::new(message, file, line);
            let mut backtrace = BACKTRACE
                .get()
                .expect("Could not get backtrace list")
                .lock();
            let index = *BACKTRACE_INDEX
                .get()
                .expect("Could not get backtrace index")
                .lock();
            backtrace[index] = Some(bte);
            *BACKTRACE_INDEX
                .get()
                .expect("Could not get backtrace index")
                .lock() = (index + 1) % MAX_BACKTRACE_ENTRIES;
        }
    }

    /// shows the last traces from the [BACKTRACE] list
    ///
    /// If serial printing is enabled it will print to the serial interface
    ///
    /// If screen printing is enabled it will print to the framebuffer
    pub fn show_trace(&self) {
        if self.serial_printing {
            serial_println!("Backtrace Info");

            let index = {
                let guard = BACKTRACE_INDEX
                    .get()
                    .expect("Could not get backtrace index")
                    .lock();
                *guard
            };

            let mut count = 0;
            let mut index = index;

            while count < MAX_BACKTRACE_ENTRIES {
                let entry = {
                    BACKTRACE
                        .get()
                        .expect("Could not get backtrace list")
                        .lock()
                        .as_ref()[index]
                };

                if let Some(entry) = entry {
                    serial_println!(
                        "{}|{}| {}",
                        entry.file,
                        entry.line,
                        core::str::from_utf8(&entry.message).unwrap_or("Invalid UTF-8"),
                    );
                }

                index = (index + 1) % MAX_BACKTRACE_ENTRIES;
                count += 1;
            }
        }

        if self.screen_printing {
            println!("Backtrace Info");

            let index = {
                let guard = BACKTRACE_INDEX
                    .get()
                    .expect("Could not get backtrace index")
                    .lock();
                *guard
            };

            let mut count = 0;
            let mut index = index;

            while count < MAX_BACKTRACE_ENTRIES {
                let entry = {
                    BACKTRACE
                        .get()
                        .expect("Could not get backtrace list")
                        .lock()
                        .as_ref()[index]
                };
                if let Some(entry) = entry {
                    println!(
                        "{}|{}| {}",
                        entry.file,
                        entry.line,
                        core::str::from_utf8(&entry.message)
                            .unwrap_or("Invalid UTF-8")
                            .trim(),
                    );
                }

                index = (index + 1) % MAX_BACKTRACE_ENTRIES;
                count += 1;
            }
        }
    }
}
