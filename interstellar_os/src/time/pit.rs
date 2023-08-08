//This file contains code for interstellar OS - https://github.com/interstellarfrog/interstellar
//Copyright (C) 2023  contributors of the interstellar OS project
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

use x86_64::instructions::{interrupts, port::Port};

const PIT_COMMAND_PORT: u16 = 0x43;
const PIT_CHANNEL_0_DATA_PORT: u16 = 0x40;
const PIT_MODE_ONESHOT: u8 = 0b00110010; // Mode 1: One-shot mode
const _PIT_MODE_RATE_GENERATOR: u8 = 0b00110100; // Mode 2: rate generator
const PIT_MODE_SQUARE_WAVE: u8 = 0b00110110; // Mode 3: Square wave generator with the given frequency
const PIT_BASE_FREQUENCY: u32 = 1193180; // The base frequency of the PIT in Hz

pub fn set_timer_interval(time_ms: u32) {
    let mut command_port = Port::new(PIT_COMMAND_PORT);
    let mut data_port = Port::new(PIT_CHANNEL_0_DATA_PORT);

    let time_seconds = time_ms as f64 / 1000.0;

    let time_freq = (1.0 / time_seconds) as u32;

    // Calculate the divide value for the desired time interval
    let divisor: u32 = PIT_BASE_FREQUENCY / time_freq;

    // Split the output
    let divisor_low: u8 = (divisor & 0xFF) as u8;
    let divisor_high: u8 = ((divisor & 0xFF00) >> 8) as u8;

    // Disable interrupts while setting up the timer
    interrupts::without_interrupts(|| {
        unsafe {
            // Set the PIT to Mode 3 (Rate generator)
            command_port.write(PIT_MODE_SQUARE_WAVE);

            // Send the low byte of the divisor
            data_port.write(divisor_low);

            // Send the high byte of the divisor
            data_port.write(divisor_high);
        }
    });
}

/// Sets the timer to tick once
pub fn set_oneshot(time_ms: u32) {
    let mut timer_port = Port::new(PIT_COMMAND_PORT);
    let mut data_port = Port::new(PIT_CHANNEL_0_DATA_PORT);

    let time_seconds = time_ms as f64 / 1000.0;

    let time_freq = (1.0 / time_seconds) as u32;

    // Calculate the divide value for the desired time interval
    let divisor: u32 = PIT_BASE_FREQUENCY / time_freq;

    // Split the output
    let divisor_low: u8 = (divisor & 0xFF) as u8;
    let divisor_high: u8 = ((divisor & 0xFF00) >> 8) as u8;

    // Disable interrupts while setting up the timer
    interrupts::without_interrupts(|| {
        // Set the PIT to one-shot mode (Mode 1)
        unsafe {
            timer_port.write(PIT_MODE_ONESHOT);

            // Send the low byte of the divisor
            data_port.write(divisor_low);

            // Send the high byte of the divisor
            data_port.write(divisor_high);
        }
    });
}
