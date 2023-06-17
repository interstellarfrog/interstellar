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

use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use conquer_once::spin::OnceCell;
use core::fmt;
use core::ptr;
use font_constants::BACKUP_CHAR;
use noto_sans_mono_bitmap::{
    get_raster, get_raster_width, FontWeight, RasterHeight, RasterizedChar,
};
use spinning_top::Spinlock;

pub enum Color {
    Red,
    Green,
    Blue,
    Black,
    White,
    Cyan,
    Magenta,
    Brown,
    LightGray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    Pink,
    Yellow,
}

impl Color {
    /// Converts the Color enum variant to a pixel format array.
    pub fn to_pixel(color: &Color, buffer_info: FrameBufferInfo) -> [u8; 4] {
        match color {
            // Both The Same For RGB/BGR
            Color::Black => [0, 0, 0, 0],
            Color::White => [255, 255, 255, 0],
            _ => {
                if buffer_info.pixel_format == PixelFormat::Rgb {
                    match color {
                        Color::Blue => [0, 0, 255, 0],
                        Color::Green => [0, 255, 0, 0],
                        Color::Cyan => [0, 255, 255, 0],
                        Color::Red => [255, 0, 0, 0],
                        Color::Magenta => [255, 0, 255, 0],
                        Color::Brown => [165, 42, 42, 0],
                        Color::LightGray => [211, 211, 211, 0],
                        Color::DarkGray => [169, 169, 169, 0],
                        Color::LightBlue => [173, 216, 230, 0],
                        Color::LightGreen => [144, 238, 144, 0],
                        Color::LightCyan => [224, 255, 255, 0],
                        Color::LightRed => [255, 99, 71, 0],
                        Color::Pink => [255, 192, 203, 0],
                        Color::Yellow => [255, 255, 0, 0],
                        _ => [0, 0, 0, 255],
                    }
                } else if buffer_info.pixel_format == PixelFormat::Bgr {
                    match color {
                        Color::Blue => [255, 0, 0, 0],
                        Color::Green => [0, 255, 0, 0],
                        Color::Cyan => [255, 255, 0, 0],
                        Color::Red => [0, 0, 255, 0],
                        Color::Magenta => [255, 0, 255, 0],
                        Color::Brown => [42, 42, 165, 0],
                        Color::LightGray => [211, 211, 211, 0],
                        Color::DarkGray => [169, 169, 169, 0],
                        Color::LightBlue => [230, 216, 173, 0],
                        Color::LightGreen => [144, 238, 144, 0],
                        Color::LightCyan => [255, 255, 224, 0],
                        Color::LightRed => [71, 99, 255, 0],
                        Color::Pink => [203, 192, 255, 0],
                        Color::Yellow => [0, 255, 255, 0],
                        _ => [0, 0, 0, 255],
                    }
                } else {
                    // Pixel Format Is Not BGR or RGB So Make Fully invisible
                    [0, 0, 0, 255]
                }
            }
        }
    }
}

pub static FRAMEBUFFER: OnceCell<Spinlock<FrameBufferWriter>> = OnceCell::uninit();

/// Prints to framebuffer
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::drivers::screen::framebuffer::_print(format_args!($($arg)*))
    };
}

/// Prints to framebuffer, appending a newline.
#[macro_export]
macro_rules! println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(
        concat!($fmt, "\n"), $($arg)*));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
        if let Some(fb) = FRAMEBUFFER.get() {
            fb.lock().write_fmt(args).unwrap()
        }
    });
}

/// Additional vertical space between lines
const LINE_SPACING: usize = 2;
/// Additional horizontal space between characters.
const LETTER_SPACING: usize = 0;

/// Padding from the border. Prevent that font is too close to border.
pub const BORDER_PADDING: usize = 1;

/// Constants for the usage of the [`noto_sans_mono_bitmap`] crate.
mod font_constants {
    use super::*;

    /// Height of each char raster. The font size is ~0.84% of this. Thus, this is the line height that
    /// enables multiple characters to be side-by-side and appear optically in one line in a natural way.
    pub const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;

    /// The width of each single symbol of the mono space font.
    pub const CHAR_RASTER_WIDTH: usize = get_raster_width(FontWeight::Regular, CHAR_RASTER_HEIGHT);

    /// Backup character if a desired symbol is not available by the font.
    /// The '�' character requires the feature "unicode-specials".
    pub const BACKUP_CHAR: char = '�';

    pub const FONT_WEIGHT: FontWeight = FontWeight::Regular;
}

/// Returns the raster of the given char or the raster of [`font_constants::BACKUP_CHAR`].
fn get_char_raster(c: char) -> RasterizedChar {
    fn get(c: char) -> Option<RasterizedChar> {
        get_raster(
            c,
            font_constants::FONT_WEIGHT,
            font_constants::CHAR_RASTER_HEIGHT,
        )
    }
    get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("Should get raster of backup char."))
}

/// Allows logging text to a pixel-based framebuffer.
pub struct FrameBufferWriter {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
    text_col: Color,
}

impl FrameBufferWriter {
    /// Creates a new logger that uses the given framebuffer.
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut logger = Self {
            framebuffer,
            info,
            x_pos: 0,
            y_pos: 0,
            text_col: Color::White,
        };
        logger.clear();
        logger
    }
    /// Sets The Y Pos To + 1 Char Height + Additional Line Spacing
    /// Then Calls self.carriage_return()
    pub fn newline(&mut self) {
        self.y_pos += font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
        self.carriage_return()
    }
    /// Resets The X Pos To Border Padding
    pub fn carriage_return(&mut self) {
        self.x_pos = BORDER_PADDING;
    }

    /// Erases all text on the screen. Resets `self.x_pos` and `self.y_pos`.
    pub fn clear(&mut self) {
        self.x_pos = BORDER_PADDING;
        self.y_pos = BORDER_PADDING;
        self.framebuffer.fill(0);
    }

    /// Returns the width of the framebuffer.
    pub fn width(&self) -> usize {
        self.info.width
    }

    /// Returns the height of the framebuffer.
    pub fn height(&self) -> usize {
        self.info.height
    }

    /// Returns the size (width, height) of the framebuffer.
    pub fn size(&self) -> (usize, usize) {
        (self.info.width, self.info.height)
    }

    /// Returns the information about the framebuffer.
    pub fn buffer_info(&mut self) -> FrameBufferInfo {
        self.info
    }

    pub fn text_line(&self) -> usize {
        self.y_pos
    }

    /// Writes a single char to the framebuffer. Takes care of special control characters, such as
    /// newlines and carriage returns.
    pub fn write_char(&mut self, c: char, color: &mut [u8; 4]) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carriage_return(),
            c => {
                let new_xpos = self.x_pos + font_constants::CHAR_RASTER_WIDTH;
                if new_xpos >= self.width() {
                    self.newline();
                }
                let new_ypos =
                    self.y_pos + font_constants::CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
                if new_ypos >= self.height() {
                    self.clear();
                }
                self.write_rendered_char(get_char_raster(c), color);
            }
        }
    }

    /// Prints a rendered char into the framebuffer.
    /// Updates `self.x_pos`.
    fn write_rendered_char(&mut self, rendered_char: RasterizedChar, color: &mut [u8; 4]) {
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                let intensity = *byte as f32 / 255.0;
                let pixel_color = [
                    (color[0] as f32 * intensity) as u8,
                    (color[1] as f32 * intensity) as u8,
                    (color[2] as f32 * intensity) as u8,
                    color[3],
                ];
                self.write_pixel(self.x_pos + x, self.y_pos + y, pixel_color);
            }
        }

        self.x_pos += rendered_char.width() + LETTER_SPACING;
    }

    /// Deletes the last character written to the framebuffer.
    pub fn delete_char(&mut self) {
        let char_width = font_constants::CHAR_RASTER_WIDTH + LETTER_SPACING;
        let line_height = font_constants::CHAR_RASTER_HEIGHT.val() + LINE_SPACING;

        // Move the cursor back by the width of the character and spacing
        if self.x_pos >= char_width {
            self.x_pos -= char_width;
        } else {
            // Move to the previous line
            if self.y_pos >= line_height {
                self.y_pos -= line_height;
            }
            // Move to the end of the previous line
            self.x_pos = self.width() - char_width;
        }

        // Clear the area of the deleted character
        let delete_color = Color::to_pixel(&Color::Black, self.info);
        self.draw_filled_rect(
            self.x_pos,
            self.y_pos,
            char_width,
            line_height,
            delete_color,
        );
    }

    pub fn write_pixel(&mut self, x: usize, y: usize, color: [u8; 4]) {
        // If Not In Screen Return
        if y >= self.height() || x >= self.width() {
            return;
        }

        // row * row size + column
        let pixel_offset = y * self.info.stride + x;
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;
        // Write Pixel To Framebuffer
        self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
            .copy_from_slice(&color[..bytes_per_pixel]);
        // Read The Pixel For Some Reason
        let _ = unsafe { ptr::read_volatile(&self.framebuffer[byte_offset]) };
    }

    /// Reads the color of a pixel at the specified coordinates.
    pub fn read_pixel_color(&self, x: usize, y: usize) -> [u8; 4] {
        if y >= self.height() || x >= self.width() {
            // Pixel coordinates are out of bounds
            return [255, 255, 255, 0];
        }

        let bytes_per_pixel = self.info.bytes_per_pixel;
        let pixel_offset = y * self.info.stride + x;
        let byte_offset = pixel_offset * bytes_per_pixel;

        // Read the color value from the framebuffer
        [
            self.framebuffer[byte_offset],
            self.framebuffer[byte_offset + 1],
            self.framebuffer[byte_offset + 2],
            0,
        ]
    }

    /// Draws a rectangle on the framebuffer.
    /// The rectangle's top-left corner is at (x, y) and its dimensions are defined by width and height.
    pub fn draw_rect(&mut self, x: usize, y: usize, width: usize, height: usize, color: [u8; 4]) {
        if y + height > self.height() || x + width > self.width() {
            return;
        }

        let bytes_per_pixel: usize = self.info.bytes_per_pixel;

        // Draw top and bottom edges
        for w in 0..width {
            let top_pixel_offset: usize = y * self.info.stride + (x + w);
            let bottom_pixel_offset: usize = (y + height - 1) * self.info.stride + (x + w);

            let top_byte_offset: usize = top_pixel_offset * bytes_per_pixel;
            let bottom_byte_offset: usize = bottom_pixel_offset * bytes_per_pixel;

            self.framebuffer[top_byte_offset..(top_byte_offset + bytes_per_pixel)]
                .copy_from_slice(&color[..bytes_per_pixel]);
            self.framebuffer[bottom_byte_offset..(bottom_byte_offset + bytes_per_pixel)]
                .copy_from_slice(&color[..bytes_per_pixel]);

            unsafe {
                ptr::read_volatile(&self.framebuffer[top_byte_offset]);
                ptr::read_volatile(&self.framebuffer[bottom_byte_offset]);
            };
        }

        // Draw left and right edges
        for h in 0..height {
            let left_pixel_offset: usize = (y + h) * self.info.stride + x;
            let right_pixel_offset: usize = (y + h) * self.info.stride + (x + width - 1);

            let left_byte_offset: usize = left_pixel_offset * bytes_per_pixel;
            let right_byte_offset: usize = right_pixel_offset * bytes_per_pixel;

            self.framebuffer[left_byte_offset..(left_byte_offset + bytes_per_pixel)]
                .copy_from_slice(&color[..bytes_per_pixel]);
            self.framebuffer[right_byte_offset..(right_byte_offset + bytes_per_pixel)]
                .copy_from_slice(&color[..bytes_per_pixel]);

            unsafe {
                ptr::read_volatile(&self.framebuffer[left_byte_offset]);
                ptr::read_volatile(&self.framebuffer[right_byte_offset]);
            };
        }
    }

    /// Draws a filled rectangle on the framebuffer.
    /// The rectangle's top-left corner is at (x, y) and its dimensions are defined by width and height.
    pub fn draw_filled_rect(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        color: [u8; 4],
    ) {
        if y + height > self.height() || x + width > self.width() {
            return;
        }

        let bytes_per_pixel: usize = self.info.bytes_per_pixel;
        for w in 0..width {
            for h in 0..height {
                let pixel_offset: usize = (y + h) * self.info.stride + (x + w);
                let byte_offset: usize = pixel_offset * bytes_per_pixel;
                self.framebuffer[byte_offset..(byte_offset + bytes_per_pixel)]
                    .copy_from_slice(&color[..bytes_per_pixel]);
                unsafe {
                    ptr::read_volatile(&self.framebuffer[byte_offset]);
                };
            }
        }
    }

    /// Draws a line on the framebuffer using Bresenham's line algorithm.
    /// The line starts from the point (from_x, from_y) and ends at the point (to_x, to_y).
    pub fn draw_line(
        &mut self,
        mut from_x: usize,
        mut from_y: usize,
        to_x: usize,
        to_y: usize,
        color: [u8; 4],
    ) {
        let dx = (to_x as isize - from_x as isize).abs();
        let dy = -(to_y as isize - from_y as isize).abs();
        let sx = if from_x < to_x { 1 } else { -1 };
        let sy = if from_y < to_y { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            self.write_pixel(from_x, from_y, color);

            if from_x == to_x && from_y == to_y {
                break;
            }

            let e2 = 2 * err;

            if e2 >= dy {
                err += dy;
                from_x = ((from_x as isize) + sx) as usize;
            }

            if e2 <= dx {
                err += dx;
                from_y = ((from_y as isize) + sy) as usize;
            }
        }
    }

    /// Draws a circle on the framebuffer using Bresenham's circle algorithm.
    /// The center of the circle is at the point (cx, cy) and its radius is defined by `radius`.
    pub fn draw_circle(&mut self, cx: usize, cy: usize, radius: usize, color: [u8; 4]) {
        let mut x = radius as isize - 1;
        let mut y = 0isize;
        let mut dx = 1isize;
        let mut dy = 1isize;
        let mut err = dx - ((radius as isize) << 1);

        while x >= y {
            self.write_pixel(
                (cx as isize + x) as usize,
                (cy as isize + y) as usize,
                color,
            );
            self.write_pixel(
                (cx as isize + y) as usize,
                (cy as isize + x) as usize,
                color,
            );
            self.write_pixel(
                (cx as isize - y) as usize,
                (cy as isize + x) as usize,
                color,
            );
            self.write_pixel(
                (cx as isize - x) as usize,
                (cy as isize + y) as usize,
                color,
            );
            self.write_pixel(
                (cx as isize - x) as usize,
                (cy as isize - y) as usize,
                color,
            );
            self.write_pixel(
                (cx as isize - y) as usize,
                (cy as isize - x) as usize,
                color,
            );
            self.write_pixel(
                (cx as isize + y) as usize,
                (cy as isize - x) as usize,
                color,
            );
            self.write_pixel(
                (cx as isize + x) as usize,
                (cy as isize - y) as usize,
                color,
            );

            if err <= 0 {
                y += 1;
                err += dy;
                dy += 2;
            }

            if err > 0 {
                x -= 1;
                dx += 2;
                err += dx - ((radius as isize) << 1);
            }
        }
    }
    /// Draws a filled circle on the framebuffer using Bresenham's circle algorithm.
    /// The center of the circle is at the point (cx, cy) and its radius is defined by `radius`.
    pub fn draw_filled_circle(&mut self, cx: usize, cy: usize, radius: usize, color: [u8; 4]) {
        let mut x = radius as isize - 1;
        let mut y = 0isize;
        let mut dx = 1isize;
        let mut dy = 1isize;
        let mut err = dx - ((radius as isize) << 1);

        while x >= y {
            for i in (cx as isize - x)..=(cx as isize + x) {
                self.write_pixel(i as usize, (cy as isize + y) as usize, color);
                self.write_pixel(i as usize, (cy as isize - y) as usize, color);
            }

            for i in (cx as isize - y)..=(cx as isize + y) {
                self.write_pixel(i as usize, (cy as isize + x) as usize, color);
                self.write_pixel(i as usize, (cy as isize - x) as usize, color);
            }

            if err <= 0 {
                y += 1;
                err += dy;
                dy += 2;
            }

            if err > 0 {
                x -= 1;
                dx += 2;
                err += dx - ((radius as isize) << 1);
            }
        }
    }

    pub fn change_text_color(&mut self, col: Color) {
        self.text_col = col;
    }
}

// Allows The Framebuffer To Be Used Between Threads
unsafe impl Send for FrameBufferWriter {}
unsafe impl Sync for FrameBufferWriter {}

impl fmt::Write for FrameBufferWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c, &mut Color::to_pixel(&self.text_col, self.info));
        }
        Ok(())
    }
}
