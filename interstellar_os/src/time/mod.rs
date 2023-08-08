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

pub mod hpet;
pub mod pit;

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

/// This is the global timer set up at OS boot
///
/// It can be used freely by most parts of the kernel
pub static mut GLOBAL_TIMER: OnceCell<Spinlock<Timer>> = OnceCell::uninit();

/// initialize the global timer
pub fn init() {
    unsafe { GLOBAL_TIMER.init_once(|| Spinlock::new(Timer::new())) };
}

#[derive(Debug, Clone, Copy)]
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
    /// Create a new Timer and records the start time
    pub fn new() -> Timer {
        let start_count = unsafe { APIC_COUNT.load(core::sync::atomic::Ordering::SeqCst) };
        Timer { start_count }
    }

    /// Get the elapsed time
    ///
    /// This is tested to be accurate within 0.3 seconds
    pub fn elapsed(&self) -> Duration {
        // get the current count say its 10
        let current_count = unsafe { APIC_COUNT.load(core::sync::atomic::Ordering::SeqCst) };
        // then we get the elapsed ticks by taking away the start count say this is 0
        let elapsed_ticks = current_count - self.start_count;
        // then we return elasped: 10 * ms per tick: 10 which would be 100ms
        Duration::from_millis(elapsed_ticks * 10) // Assuming that the timer increments every 10ms
    }

    /// Suspend the CPU for a [Duration]
    ///
    /// This is tested to be accurate within 0.3 seconds
    ///
    /// The lowest time you can give is 10ms anything under this will finish after 1 loop cycle
    pub fn sleep(self, duration: Duration) {
        // Calculate the number of timer ticks for the desired duration

        // Assuming that the timer increments every 10ms
        // Assuming duration.as_millis is 1000 then it = 100
        let dur = duration.as_millis() / 10;

        // Get the current timer count
        // Assuming this is 0
        let start_count = unsafe { APIC_COUNT.load(core::sync::atomic::Ordering::SeqCst) };

        // Calculate the target count
        // Assuming this is 100
        let target_count = start_count + dur as u64;

        // Wait until the timer count reaches the target
        loop {
            let current_count = unsafe { APIC_COUNT.load(core::sync::atomic::Ordering::SeqCst) };
            // Then assuming this would be if 0 >= 100    - wait for 100 ticks which is 1000ms which is 1s
            if current_count >= target_count {
                break;
            }
        }
    }
}
