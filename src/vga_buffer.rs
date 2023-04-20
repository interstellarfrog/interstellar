#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // Store As u8
pub enum Color { // Posible Colors Enum
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

impl ColorCode { // Returns The ColorCode For The Given Background and Foreground Colors
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
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT], 
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer, // Static Reference To The Buffer
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte { // Match The Byte
            b'\n' => self.new_line(), // If The Byte Is New Line Byte Call The New Line Function
            byte => { // Else
                if self.column_position >= BUFFER_WIDTH { // If The Column Position Is Greater Than The Buffer Width
                    self.new_line(); // Then Move To The New Line
                }

                let row = BUFFER_HEIGHT - 1;   // Get The Row Position
                let col = self.column_position; // Get The Column Position

                let color_code = self.color_code; // Get The Color Code
                self.buffer.chars[row][col] = ScreenChar { // Set The Character In The Buffer
                    ascii_character: byte,                      // The Byte
                    color_code,                                // The Color Code
                };
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


    fn new_line(&mut self) { todo!() }
}


pub fn print_something() {
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_byte(b'H');
    writer.write_string("ello ");
    writer.write_string("Wörld!");
}