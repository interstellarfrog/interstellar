use alloc::string::String;
use core::fmt;
use core::fmt::{Result, Write};
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;
use x86_64::instructions::interrupts;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // Store As u8
pub enum Color {
    // Posible Colors Enum
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    // Returns The ColorCode For The Given Background and Foreground Colors
    fn new(foreground: Color, background: Color) -> ColorCode {
        // Bitwise OR of the Background and Foreground Colors
        // Say If We Want To Do Black which = 0 and in binary = 0000 And White = 15 = 1111
        // (0000) (1111) Shift The Bits Left By 4 into 0000 Which Is 00001111 Or 0x0F
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // C Style Struct - Field Ordering
pub struct ScreenChar {
    pub ascii_character: u8,
    pub color_code: ColorCode,
}

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer {
    pub chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT], // Make This Volatile - Dont Optimize This Away
}

pub struct Writer {
    column_position: usize,
    row: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer, // Static Reference To The Buffer
    rainbow: bool, // For Rainbow Mode In Command Line
    rainbow_index: usize, // For Rainbow Mode
}
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row: 0,
        color_code: ColorCode::new(Color::LightBlue, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        rainbow: false, // Wether Rainbow Mode Is Enabled Or Not
        rainbow_index: 0, // The Color Index Rainbow Mode Uses
    });
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            // Match The Byte
            b'\n' => self.new_line(), // If The Byte Is New Line Byte Call The New Line Function
            byte => {
                // Else
                if self.column_position >= BUFFER_WIDTH {
                    // If The Column Position Is Greater Than The Buffer Width
                    self.new_line(); // Then Move To The New Line
                }

                let row = self.row;

                let col = self.column_position; // Get The Column Position

                if self.rainbow {

                    if self.rainbow_index == 0 {
                        self.rainbow_index += 1;
                    }

                    self.buffer.chars[row][col].write(ScreenChar {
                        ascii_character: byte,
                        color_code: ColorCode((0 as u8) << 4 | (self.rainbow_index as u8)),
                    });
                    self.rainbow_index += 1;
                    if self.rainbow_index == 16 {
                        self.rainbow_index = 0;
                    }
                } else {
                    let color_code = self.color_code;

                    self.buffer.chars[row][col].write(ScreenChar {
                        // Volatile Write
                        // Set The Character In The Buffer
                        ascii_character: byte, // The Byte
                        color_code,            // The Color Code
                    });
                }

                self.column_position += 1; // Increment The Column Position After Writing The Character
            }
        }
    }

    pub fn delete_char(&mut self) {
        let row = self.row;

        let col = self.column_position;

        self.buffer.chars[row][col - 1].write(ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        });
        self.column_position -= 1;
    }

    pub fn read_line(&mut self) {
        let mut line = String::from("");

        let row = self.row;

        for i in 0..BUFFER_WIDTH {
            let x = self.buffer.chars[row][i].read();
            line.push(char::from(x.ascii_character));
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte), // If The Byte Is Between 0x20 And 0x7E Or New Line Byte
                _ => self.write_byte(0xfe), // Else Write The Byte 0xFE - INVALID CHARACTER
            }
        }
    }

    pub fn new_line(&mut self) {
        // If We Have Reached The End Of The Screen
        if self.row == BUFFER_HEIGHT - 1 {

            // We Want To Shift Every Line Up Instead
            let row_end = BUFFER_HEIGHT;
            let row_start = 1;

            // Move Every Row up By One From The Bottom Except From The 0th Because That Is Overwriten
            for row in row_start..row_end {
                // Row 1..24
                for col in 0..BUFFER_WIDTH {
                    // Col 0..25
                    let character = self.buffer.chars[row][col].read(); // Row 1 col 0

                    // Get row_Pos To Move Char To - 1 Pos which is up
                    let pos = row - 1;

                    self.buffer.chars[pos][col].write(character); // Write Char To One Line Off - 1
                }
            }

            let clear_row = self.row;
            self.clear_row(clear_row);
        } else {
            self.row += 1;
        }

        self.column_position = 0;
    }
    pub fn clear_row(&mut self, row: usize) {
        // Does What It Says
        let blank = ScreenChar {
            // Blank ScreenChar
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            // For Every Char In Given Row
            self.buffer.chars[row][col].write(blank) // Write Blank Char
        }
    }

    pub fn clear_screen_chars(&mut self) {
        let blank = ScreenChar {
            // Blank ScreenChar
            ascii_character: b' ',
            color_code: self.color_code,
        };

        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row][col].write(blank)
            }
        }
        self.write_string("Neutron> ");
    }

    pub fn return_row(&mut self) -> usize {
        return self.row;
    }
    pub fn rainbow_toggle(&mut self) {
        if self.rainbow {
            self.rainbow = false;
        } else {
            self.rainbow = true;
        }
    }

}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> Result {
        // Formatted
        self.write_string(s);
        Ok(())
    }
}
//--------------------------------------------------------------------------

#[macro_export]
macro_rules! print { // Calls _print after formatting
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println { // Calls print! macro but adds newline
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        // Stops DeadLocks
        WRITER.lock().write_fmt(args).unwrap();
    });
}

//--------------------------------------------------------------------------

#[test_case]
fn test_println_simple() {
    println!("test_println_simple output");
}

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    let s = "Some Test String That Fits On A Single Line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });
}
