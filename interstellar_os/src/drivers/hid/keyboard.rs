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

use crate::other::log::LOGGER;

pub fn init() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Initializing keyboard"), file!(), line!());

    LOGGER.get().unwrap().lock().info("Initializing keyboard");

    let mut cmd = x86_64::instructions::port::Port::<u8>::new(0x64);
    unsafe {
        cmd.write(0xae); // enable keyboard port
    }
}
