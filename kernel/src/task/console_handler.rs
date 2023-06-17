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

use crate::drivers::screen::framebuffer::BORDER_PADDING;
use crate::FRAMEBUFFER;
use crate::{console::handle_console, print};
use alloc::string::String;
use futures_util::stream::StreamExt;
use pc_keyboard::{layouts, DecodedKey, HandleControl, KeyCode, KeyEvent, Keyboard, ScancodeSet1};

use super::keyboard::ScancodeStream;

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
                handle_console(input_buffer.as_str());
                input_buffer.clear();
                let line = FRAMEBUFFER.get().unwrap().lock().text_line(); // Putting Into Variable Fixes Deadlock
                if line == BORDER_PADDING {
                    print!("Neutron> ");
                } else {
                    print!("\nNeutron> ");
                }
            } else if key_event == KeyEvent::new(KeyCode::Backspace, pc_keyboard::KeyState::Down) {
                if !input_buffer.is_empty() {
                    FRAMEBUFFER.get().unwrap().lock().delete_char();
                    input_buffer.pop();
                }
            } else if let Some(key) = keyboard.process_keyevent(key_event) {
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
