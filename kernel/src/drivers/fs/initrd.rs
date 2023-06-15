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

use crate::{drivers::fs::fat32::Fat32Reader, println};



pub fn read_initrd(initrd_ptr: *const u8, initrd_size: u64) {
    let initrd_slice = unsafe { core::slice::from_raw_parts(initrd_ptr, initrd_size as usize) };

    if let Some(fat32_reader) = Fat32Reader::new(initrd_slice) {
        let file_names = fat32_reader.get_file_names();
        
        for file_name in file_names {
            println!("File: {}", file_name);
            if let Some(file_data) = fat32_reader.read_file(&file_name) {
                // Process the file data
                println!("File Contents: {:?}", file_data);
            } else {
                println!("Error reading file: {}", file_name);
            }
        }
    } else {
        println!("Invalid FAT Initrd")
    }
}

