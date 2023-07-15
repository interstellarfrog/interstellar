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

// Define an enum to represent different types of events
enum Event {
    KeyPress(char),
    MouseMove(u32, u32),
}

// Window Manager struct
pub struct WindowManager {
    event_queue: [Event; QUEUE_SIZE],
    queue_start: usize,
    queue_end: usize,
}

impl WindowManager {
    // Method to add an event to the event queue
    fn add_event(&mut self, event: Event) {
        self.event_queue[self.queue_end] = event;
        self.queue_end = (self.queue_end + 1) % QUEUE_SIZE;
    }

    // Method to process events from the event queue
    fn process_events(&mut self) {
        while self.queue_start != self.queue_end {
            let event = &self.event_queue[self.queue_start];
            self.queue_start = (self.queue_start + 1) % QUEUE_SIZE;

            // Handle different event types
            match event {
                Event::KeyPress(key) => {
                    // Process key press event
                }
                Event::MouseMove(x, y) => {
                    // Process mouse move event
                } // Handle other event types
            }
        }
    }
}

// Constants
const QUEUE_SIZE: usize = 100; // Adjust the size as per your needs
