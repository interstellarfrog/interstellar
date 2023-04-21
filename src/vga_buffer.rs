use volatile::Volatile;
use core::fmt;
use core::fmt::{ Write, Result};
use lazy_static::lazy_static;
use spin::Mutex;



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
struct ColorCode(u8);

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
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT], // Make This Volatile - Dont Optimize This Away
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer, // Static Reference To The Buffer
}
lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
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

                let row = BUFFER_HEIGHT - 1; // Get The Row Position
                let col = self.column_position; // Get The Column Position

                let color_code = self.color_code; // Get The Color Code
                self.buffer.chars[row][col].write(ScreenChar { // Volatile Write
                    // Set The Character In The Buffer
                    ascii_character: byte, // The Byte
                    color_code,            // The Color Code
                });
                self.column_position += 1; // Increment The Column Position After Writing The Character
            }
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

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT { // Row 0 Is Moved Off Screen So No Need To Count This
            for col in 0..BUFFER_WIDTH { // For Every Char
                let character = self.buffer.chars[row][col].read(); // Read Char Pos
                self.buffer.chars[row - 1][col].write(character); // Write Char To One Line Up
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }
    fn clear_row(&mut self, row: usize) { // Does What It Says
        let blank = ScreenChar { // Blank ScreenChar
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH { // For Every Char In Given Row
            self.buffer.chars[row][col].write(blank) // Write Blank Char
        }
    }
}

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> Result { // Formatted
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
    WRITER.lock().write_fmt(args).unwrap();
}