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

use crate::other::console::handle_console;
use crate::print;
use crate::FRAMEBUFFER;
use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;
use conquer_once::spin::OnceCell;
use futures_util::stream::StreamExt;
use pc_keyboard::{layouts, DecodedKey, HandleControl, KeyCode, KeyEvent, Keyboard, ScancodeSet1};
use spinning_top::Spinlock;

use super::keyboard::ScancodeStream;

/// Prints to framebuffer
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::task::console_handler::_print(format_args!($($arg)*))
    };
}

/// Prints to framebuffer, appending a newline.
#[macro_export]
macro_rules! println {
    () => ($crate::task::console_handler::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::print!(
        concat!($fmt, "\n"), $($arg)*));
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // WARNING! This code is really complex because of \n

    // We need to add lines to CONSOLE_INFO
    // So we can make the console scrollable

    // Possible problem values we need to handle
    //
    // \n
    // \n\n\n - could be any number of \n in a row
    // text\n
    // text\n\n - could be any number of \n in a row
    // \ntext
    // \n\ntext - could be any number of \n in a row
    // \ntext\n
    // \n\ntext\n\n - could be any number of \n in a row

    let mut lines = alloc::format!("{}", args.clone());
    let buffer_width = FRAMEBUFFER.get().unwrap().lock().info.width;
    let char_width = crate::drivers::screen::framebuffer::font_constants::CHAR_RASTER_WIDTH
        + crate::drivers::screen::framebuffer::LETTER_SPACING;
    let max_chars_in_line = buffer_width / char_width;

    let binding = lines.clone();
    let chars_with_new_lines = binding.chars().enumerate().map(|(i, c)| {
        if i % max_chars_in_line == 0 && i != lines.len() - 1 {
            format!("{}\n", c)
        } else {
            format!("{}", c)
        }
    });

    lines = chars_with_new_lines.collect::<String>();

    if lines.as_str() == "\n" {
        CONSOLE_INFO.get().unwrap().lock().add_new_line();
        CONSOLE_INFO
            .get()
            .unwrap()
            .lock()
            .add_to_current_line("".to_string());

        interrupts::without_interrupts(|| {
            if let Some(fb) = FRAMEBUFFER.get() {
                fb.lock().write_fmt(args).unwrap()
            }
        });

        return;
    }

    let mut last_char_new_line = false;
    let mut new_line_count = 0;
    let mut new_line_double_count = 0;
    let mut new_line_double_list: Vec<(u64, u8)> = Vec::new();
    let mut last_line_single = false;
    let mut first_line_single = false;

    if lines.ends_with('\n') {
        last_line_single = true;
    }

    if lines.starts_with('\n') {
        first_line_single = true;
    }

    for (i, c) in lines.chars().enumerate() {
        // Generates a list that contains the "new line" number at which a double new line occurs and the amount new lines it finds
        if c == '\n'
            && last_char_new_line
            && new_line_double_count < 100 // Should never need more than 100 new lines in a row 
            && new_line_count < u64::MAX
        // should never need more than 18446744073709551615 new lines in a console input or output
        {
            new_line_count += 1;
            new_line_double_count += 1;
            if lines.ends_with('\n')
                && i == lines.clone().trim_end_matches('\n').chars().count() + 1
            {
                last_line_single = false;
            }

            if lines.starts_with('\n') && i == 1 {
                first_line_single = false;
            }

            if i + 1 < lines.chars().count() {
                let next_char = lines.chars().nth(i + 1).unwrap();
                if next_char != '\n' {
                    new_line_double_list
                        .append(&mut alloc::vec![(new_line_count, new_line_double_count)]);
                }
            } else {
                new_line_double_list
                    .append(&mut alloc::vec![(new_line_count, new_line_double_count)]);
            }
        } else if c == '\n' {
            last_char_new_line = true;
            new_line_count += 1;
            new_line_double_count = 0;
        } else {
            last_char_new_line = false;
            new_line_double_count = 0;
        }
    }

    if lines.as_str().contains('\n') {
        // If there is possible multiple lines (it could just end/start in a \n)
        let mut number_of_lines_in_middle = 0;
        let mut number_of_new_line_on_end = 0;

        // Check if we should jump to a new line
        if lines.as_str().starts_with('\n') {
            CONSOLE_INFO.get().unwrap().lock().add_new_line();
            CONSOLE_INFO
                .get()
                .unwrap()
                .lock()
                .add_to_current_line("".to_string());

            if !first_line_single && !new_line_double_list.is_empty() {
                let (_, number_of_double_new_lines) = new_line_double_list.clone()[0];

                for _ in 1..=number_of_double_new_lines {
                    CONSOLE_INFO.get().unwrap().lock().add_new_line(); // Make new lines for the number of double new lines
                    CONSOLE_INFO
                        .get()
                        .unwrap()
                        .lock()
                        .add_to_current_line("".to_string());
                }
                if number_of_double_new_lines as usize + 1 == lines.chars().count() {
                    // If there is the same amount of new lines as the whole string
                    interrupts::without_interrupts(|| {
                        if let Some(fb) = FRAMEBUFFER.get() {
                            fb.lock().write_fmt(args).unwrap()
                        }
                    });

                    return;
                }
            }
        }

        // Check if we should add the last line to current line or create a new line for it
        if lines.as_str().ends_with('\n') {
            number_of_new_line_on_end += 1;

            if !new_line_double_list.is_empty() && !last_line_single {
                let (_, number_of_double_new_lines) =
                    new_line_double_list.clone()[new_line_double_list.len() - 1];
                number_of_new_line_on_end += number_of_double_new_lines;
            }
        }

        if lines
            .clone()
            .trim_end_matches('\n')
            .trim_start_matches('\n')
            .contains('\n')
        {
            // Calculate how many lines there is
            #[allow(clippy::explicit_counter_loop)]
            for _ in lines
                .clone()
                .trim_end_matches('\n')
                .trim_start_matches('\n')
                .split('\n')
            {
                number_of_lines_in_middle += 1; // Number of lines in middle of string
            }
        }

        if !lines.as_str().ends_with('\n') {
            if number_of_lines_in_middle == 0 {
                CONSOLE_INFO
                    .get()
                    .unwrap()
                    .lock()
                    .add_to_current_line(lines.clone().replace('\n', ""));
            } else {
                for (i, line) in lines
                    .trim_end_matches('\n')
                    .trim_start_matches('\n')
                    .split('\n')
                    .enumerate()
                {
                    if i + 1 == number_of_lines_in_middle {
                        CONSOLE_INFO
                            .get()
                            .unwrap()
                            .lock()
                            .add_to_current_line(line.to_string());
                    } else {
                        // for example if 4 < 4   - dont save the last line as it can be changed
                        CONSOLE_INFO
                            .get()
                            .unwrap()
                            .lock()
                            .add_to_current_line(line.to_string()); // Every time we print to the screen save the line

                        CONSOLE_INFO.get().unwrap().lock().add_new_line();
                    }
                }
            }
        } else if number_of_lines_in_middle > 0 {
            for (count, line) in lines
                .trim_end_matches('\n')
                .trim_start_matches('\n')
                .split('\n')
                .enumerate()
            {
                CONSOLE_INFO
                    .get()
                    .unwrap()
                    .lock()
                    .add_to_current_line(line.to_string());
                if count + 1
                    != lines
                        .trim_end_matches('\n')
                        .trim_start_matches('\n')
                        .split('\n')
                        .count()
                {
                    // If its not the last line in the list
                    CONSOLE_INFO.get().unwrap().lock().add_new_line();
                    CONSOLE_INFO
                        .get()
                        .unwrap()
                        .lock()
                        .add_to_current_line("".to_string());
                }
            }

            if !last_line_single {
                for _ in 1..=number_of_new_line_on_end {
                    CONSOLE_INFO.get().unwrap().lock().add_new_line();
                    CONSOLE_INFO
                        .get()
                        .unwrap()
                        .lock()
                        .add_to_current_line("".to_string());
                }
            } else if last_line_single {
                CONSOLE_INFO.get().unwrap().lock().add_new_line();
                CONSOLE_INFO
                    .get()
                    .unwrap()
                    .lock()
                    .add_to_current_line("".to_string());
            }
        } else {
            CONSOLE_INFO.get().unwrap().lock().add_to_current_line(
                lines
                    .trim_end_matches('\n')
                    .trim_start_matches('\n')
                    .to_string(),
            );

            if !last_line_single {
                for _ in 1..=number_of_new_line_on_end {
                    CONSOLE_INFO.get().unwrap().lock().add_new_line();
                    CONSOLE_INFO
                        .get()
                        .unwrap()
                        .lock()
                        .add_to_current_line("".to_string());
                }
            } else if last_line_single {
                CONSOLE_INFO.get().unwrap().lock().add_new_line();
                CONSOLE_INFO
                    .get()
                    .unwrap()
                    .lock()
                    .add_to_current_line("".to_string());
            }
        }
    } else {
        // no \n found - this means we do not create a new line for the next iteration, we add to this line
        CONSOLE_INFO
            .get()
            .unwrap()
            .lock()
            .add_to_current_line(lines);
    }

    interrupts::without_interrupts(|| {
        if let Some(fb) = FRAMEBUFFER.get() {
            fb.lock().write_fmt(args).unwrap()
        }
    });
}

pub fn init() {
    CONSOLE_INFO.init_once(|| {
        let console_lines: Vec<String> = vec![];

        spinning_top::Spinlock::new(ConsoleInfo {
            console_lines,
            max_lines: 400,
            current_line_index: 0,
        })
    });
}

/// Provides info about the console
pub static CONSOLE_INFO: OnceCell<Spinlock<ConsoleInfo>> = OnceCell::uninit();

pub struct ConsoleInfo {
    /// The lines of each input
    pub console_lines: Vec<String>,
    /// The max number of lines the console_lines can hold before removing past lines
    pub max_lines: u64,
    /// Current line index for console lines
    pub current_line_index: usize,
}

impl ConsoleInfo {
    /// Sets max number of lines [ConsoleInfo] holds
    pub fn set_max_lines(&mut self, max_lines: u64) {
        if max_lines > 20 {
            self.max_lines = max_lines
        }
    }

    /// Gets a console line stored in [ConsoleInfo]
    pub fn get_console_line(&self, line_index: usize) -> Option<String> {
        if line_index <= self.console_lines.len() + 1 {
            return Some(self.console_lines[line_index].clone());
        }
        None
    }

    /// Adds a totally new line and increments the current_line_index by 1
    ///
    /// Meaning this line cannot be appended to
    pub fn add_console_line(&mut self, line: String) {
        if self.console_lines.len() as u64 >= self.max_lines {
            self.console_lines[0].pop();
        }
        self.console_lines.append(&mut alloc::vec![line]);
        self.add_new_line();
    }

    /// Increments the current_line_index by 1
    pub fn add_new_line(&mut self) {
        if self.current_line_index as u64 + 1_u64 < self.max_lines {
            self.current_line_index += 1;
        } else {
            self.console_lines[0].pop();
            self.current_line_index = (self.max_lines - 1) as usize;
        }
    }

    /// Adds to the current line using the current_line_index
    ///
    /// This line can be appended to by calling this multiple times
    ///
    /// If the line is missing it creates a new one
    pub fn add_to_current_line(&mut self, line: String) {
        if self.console_lines.len() == self.current_line_index + 1 {
            // If current line exists
            self.console_lines[self.current_line_index] =
                self.console_lines[self.current_line_index].to_owned() + &line;
        } else {
            if (self.current_line_index + 1) as u64 >= self.max_lines {
                self.console_lines[0].pop();
            }
            // else create line
            self.console_lines.append(&mut alloc::vec![line]);
        }
    }
}

pub async fn console_start() {
    print!("Neutron> ");
    let mut scancode_stream = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Uk105Key,
        HandleControl::Ignore,
    );
    let mut input_buffer = String::new();

    while let Some(scancode) = scancode_stream.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if key_event == KeyEvent::new(KeyCode::Return, pc_keyboard::KeyState::Down) {
                // If not showing the bottom line jump down to it

                let no_new_line = handle_console(input_buffer.as_str());
                input_buffer.clear();

                if no_new_line {
                    print!("Neutron> ")
                } else {
                    print!("\nNeutron> ");
                }
            } else if key_event == KeyEvent::new(KeyCode::Backspace, pc_keyboard::KeyState::Down) {
                if !input_buffer.is_empty() {
                    if !CONSOLE_INFO.get().unwrap().lock().console_lines.is_empty() {
                        let current_index = CONSOLE_INFO.get().unwrap().lock().current_line_index;
                        let line_len = CONSOLE_INFO.get().unwrap().lock().console_lines
                            [current_index]
                            .chars()
                            .count();
                        CONSOLE_INFO.get().unwrap().lock().console_lines[current_index]
                            .remove(line_len - 1);
                    }

                    FRAMEBUFFER.get().unwrap().lock().delete_char();
                    input_buffer.pop();
                }
            } else if key_event == KeyEvent::new(KeyCode::ArrowUp, pc_keyboard::KeyState::Down) {
                // If possible move the lines one down and draw the last line from the list at the top
            } else if let Some(key) = keyboard.process_keyevent(key_event) {
                // If not showing the bottom line jump down to it
                match key {
                    DecodedKey::Unicode(character) => {
                        if character == '\n' {
                            handle_console(input_buffer.as_str());
                            input_buffer.clear();
                            print!("\nNeutron> ");
                        } else {
                            print!("{}", character);
                            input_buffer.push(character);
                        }
                    }
                    DecodedKey::RawKey(_raw_key) => {}
                }
            }
        }
    }
}
