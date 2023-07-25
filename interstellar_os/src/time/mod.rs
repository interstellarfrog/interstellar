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
use spinning_top::Spinlock;

pub static TIMER: OnceCell<Spinlock<Timer>> = OnceCell::uninit();

pub fn init() {
    TIMER.init_once(|| Spinlock::new(Timer { count: 0 }));
}

pub struct Timer {
    count: u64,
}

impl Timer {
    /// Increment the count by 1
    pub fn tick(&mut self) {
        self.count += 1;
    }
    // Returns the current timer count
    pub fn get_count(&self) -> u64 {
        self.count
    }
}
