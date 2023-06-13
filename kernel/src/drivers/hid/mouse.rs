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

use core::sync::atomic::{
    AtomicUsize, 
    Ordering, 
    AtomicBool
};

use alloc::{
    sync::Arc, 
    boxed::Box
};

use crate::{drivers::screen::framebuffer::{Color, FRAMEBUFFER}
};

use ps2_mouse::{
    Mouse as MouseDevice, 
    MouseState
};

use conquer_once::spin::OnceCell;
use crossbeam_utils::atomic::AtomicCell;
use spin::Mutex;
use spinning_top::Spinlock;

static MOUSE: OnceCell<Mouse> = OnceCell::uninit();

const CURSOR: [u8; 170] = [
    1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 1, 0, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 1, 0, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 1, 0, 0, 0, 0, 0,
    1, 2, 2, 2, 2, 1, 0, 0, 0, 0,
    1, 2, 2, 2, 2, 2, 1, 0, 0, 0,
    1, 2, 2, 2, 2, 2, 2, 1, 0, 0,
    1, 2, 2, 2, 2, 2, 2, 2, 1, 0,
    1, 2, 2, 2, 2, 2, 2, 2, 2, 1,
    1, 2, 2, 2, 2, 2, 1, 1, 1, 1,
    1, 2, 2, 2, 2, 2, 1, 0, 0, 0,
    1, 2, 2, 1, 1, 2, 2, 1, 0, 0,
    1, 2, 1, 0, 1, 2, 2, 1, 0, 0,
    1, 1, 0, 0, 0, 1, 2, 2, 1, 0,
    0, 0, 0, 0, 0, 1, 2, 2, 1, 0,
    0, 0, 0, 0, 0, 0, 1, 1, 0, 0,
];
const CURSOR_SIZE: usize = 170;
const CURSOR_ROW_SIZE: u8 = 10;

/// Initialize the mouse.
pub fn init() {
    MOUSE.init_once(Mouse::default);
}
/// Get the mouse instance.
pub(crate) fn get() -> Option<Mouse> {
    MOUSE.get().cloned()
}

/// Mouse struct representing a mouse device.
#[derive(Clone)]
pub(crate) struct Mouse {
    dev: Arc<Spinlock<MouseDevice>>,
    state: Arc<AtomicCell<Option<MouseState>>>,
    x: Arc<AtomicUsize>,
    y: Arc<AtomicUsize>,
    last_x: Arc<AtomicUsize>,
    last_y: Arc<AtomicUsize>,
    first_draw: Arc<AtomicBool>,
    last_pixel_pos_and_colors: Arc<Mutex<AtomicCell<Option<Box<[[usize; 6]; CURSOR_SIZE]>>>>>,
}
impl Default for Mouse {
    fn default() -> Self {
        let mut cmd = x86_64::instructions::port::Port::<u8>::new(0x64);
        let mut data = x86_64::instructions::port::Port::<u8>::new(0x60);
        unsafe {
            cmd.write(0xa8); // enable aux port
            cmd.write(0x20); // read command byte
            let mut status = data.read();
            status |= 0b10; // enable aux port interrupts
            cmd.write(0x60); // write command byte
            data.write(status);
            cmd.write(0xd4); // signal that next write is to mouse input buffer
            data.write(0xf4); // enable mouse
        }

        let mut dev = MouseDevice::default();
        dev.set_on_complete(Self::handler);

        Self {
            dev: Arc::new(Spinlock::new(dev)),
            state: Default::default(),
            x: Default::default(),
            y: Default::default(),
            last_x: Default::default(),
            last_y: Default::default(),
            first_draw: Arc::new(AtomicBool::new(true)),
            last_pixel_pos_and_colors: Default::default(),
        }
    }
}


impl Mouse {
    /// Mouse handler function called when a mouse event occurs.
    fn handler(state: MouseState) {
        let mouse_obj: &Mouse = MOUSE.get().expect("mouse not initialized");
        let buffer_info = FRAMEBUFFER.get().unwrap().lock().buffer_info();

        let x = mouse_obj.x.load(Ordering::Relaxed);
        let y = mouse_obj.y.load(Ordering::Relaxed);

        let first_draw = mouse_obj.first_draw.load(Ordering::Relaxed);

        // If it's not the first draw, restore pixels
        if !first_draw {
            let old_pix_cols = mouse_obj.last_pixel_pos_and_colors.lock().take().unwrap();
            // Restore pixels at old cursor position
            for i in 0..CURSOR_SIZE {
                let [pixel_x, pixel_y, pixel_color1, pixel_color2, pixel_color3, pixel_color4] = old_pix_cols[i];
                let pixel_color = [pixel_color1 as u8, pixel_color2 as u8, pixel_color3 as u8, pixel_color4 as u8];
                FRAMEBUFFER
                    .get()
                    .unwrap()
                    .lock()
                    .write_pixel(pixel_x, pixel_y, pixel_color);
            }
        }

        // Save pixels at the new cursor position
        let mut required_pixels_to_write_back: [[usize; 6]; CURSOR_SIZE] = [[0; 6]; CURSOR_SIZE];

        for (i, &_pix) in CURSOR.iter().enumerate() {
            let temp_x = x + (i % CURSOR_ROW_SIZE as usize);
            let temp_y = y + (i / CURSOR_ROW_SIZE as usize);

            let pixel_color = FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .read_pixel_color(temp_x, temp_y);
            required_pixels_to_write_back[i] = [
                temp_x,
                temp_y,
                pixel_color[0] as usize,
                pixel_color[1] as usize,
                pixel_color[2] as usize,
                pixel_color[3] as usize,
            ];
        }

        // Draw cursor at the new position
        for (i, &pix) in CURSOR.iter().enumerate() {
            let temp_x = x + (i % CURSOR_ROW_SIZE as usize);
            let temp_y = y + (i / CURSOR_ROW_SIZE as usize);

            if pix == 1 {
                FRAMEBUFFER
                    .get()
                    .unwrap()
                    .lock()
                    .write_pixel(temp_x, temp_y, Color::to_pixel(&Color::Black, buffer_info));
            } else if pix == 2 {
                FRAMEBUFFER
                    .get()
                    .unwrap()
                    .lock()
                    .write_pixel(temp_x, temp_y, Color::to_pixel(&Color::White, buffer_info));
            }
        }

        mouse_obj.last_pixel_pos_and_colors.lock().store(Some(Box::new(required_pixels_to_write_back)));
        mouse_obj.state.store(Some(state));
        mouse_obj.set_pos();
        mouse_obj.last_x.store(x, Ordering::Relaxed);
        mouse_obj.last_y.store(y, Ordering::Relaxed);
        mouse_obj.first_draw.store(false, Ordering::Relaxed);
    }

    /// Add a mouse packet to the mouse device.
    ///
    /// # Arguments
    ///
    /// * `packet` - The mouse packet to be processed.
    pub async fn add(&self, packet: u8) {
        let mut dev = self.dev.lock();
        dev.process_packet(packet);
    }

    /// Set the current position of the mouse based on the state.
    fn set_pos(&self) {
        let state = self.state.load().expect("mouse state not initialized");
        let dx = state.get_x();
        let dy = state.get_y();

        // Update the x-coordinate of the mouse position
        if dx > 0 {
            self.x.fetch_add(dx as usize, Ordering::Relaxed);
        } else {
            self.x.fetch_sub(dx.unsigned_abs() as usize, Ordering::Relaxed);
        }

        // Update the y-coordinate of the mouse position
        if dy > 0 {
            self.y.fetch_sub(dy as usize, Ordering::Relaxed);
        } else {
            self.y.fetch_add(dy.unsigned_abs() as usize, Ordering::Relaxed);
        }

        // Limit the mouse position to the screen boundaries
        self.limit_pos();
    }

    /// Limit the mouse position to the boundaries of the screen.
    fn limit_pos(&self) {
        let x = self.x.load(Ordering::Relaxed);
        let y = self.y.load(Ordering::Relaxed);
        let (width, height) = crate::drivers::screen::framebuffer::FRAMEBUFFER.get().unwrap().lock().size();

        // Ensure the x-coordinate is within the screen width
        if x > width - 1 {
            self.x.store(width - 1, Ordering::Relaxed);
        }

        // Ensure the y-coordinate is within the screen height
        if y > height - 1 {
            self.y.store(height - 1, Ordering::Relaxed);
        }
    }
}