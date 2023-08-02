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

use conquer_once::spin::OnceCell;
use core::{sync::atomic::AtomicU64, time::Duration};
use spinning_top::Spinlock;

/// This is the timer count for the PIT timer this is incremented every tick
///
/// # Fun Fact
///
/// This will take approx 5.85 Billion years to overflow at a 10ms per second tick
pub static mut PIT_COUNT: AtomicU64 = AtomicU64::new(0);

/// This is the timer count for the LAPIC timer this is incremented every tick
///
/// # Fun Fact
///
/// This will take approx 5.85 Billion years to overflow at a 10ms per second tick
pub static mut APIC_COUNT: AtomicU64 = AtomicU64::new(0);

pub static mut GLOBAL_TIMER: OnceCell<Spinlock<Timer>> = OnceCell::uninit();

/// initialize the global timer
pub fn init() {
    unsafe { GLOBAL_TIMER.init_once(|| Spinlock::new(Timer::new())) };
}

pub struct Timer {
    /// This is the start timer count for the timer
    ///
    /// # Fun Fact
    ///
    /// This will take approx 5.85 Billion years to overflow at a 10ms per second tick
    start_count: u64,
}

impl Timer {
    #[allow(clippy::new_without_default)]
    // Create a new Timer and record the start time
    pub fn new() -> Timer {
        let start_count = unsafe { APIC_COUNT.load(core::sync::atomic::Ordering::SeqCst) };
        Timer { start_count }
    }

    // Get the elapsed time in milliseconds
    pub fn elapsed(&self) -> Duration {
        let current_count = unsafe { APIC_COUNT.load(core::sync::atomic::Ordering::SeqCst) };
        let elapsed_ticks = current_count - self.start_count;
        Duration::from_millis(elapsed_ticks * 10) // Assuming that the timer increments every 10ms
    }

    pub fn sleep(self, duration: Duration) {
        // Calculate the number of timer ticks for the desired duration
        let ticks = duration.as_millis() / 10; // Assuming that the timer increments every 10ms

        // Get the current timer count
        let start_count = unsafe { APIC_COUNT.load(core::sync::atomic::Ordering::SeqCst) };

        // Calculate the target count
        let target_count = start_count + ticks as u64;

        // Wait until the timer count reaches the target
        loop {
            let current_count = unsafe { APIC_COUNT.load(core::sync::atomic::Ordering::SeqCst) };
            if current_count >= target_count {
                break;
            }
        }
    }
}
