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
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use conquer_once::spin::OnceCell;
use spinning_top::Spinlock;

pub struct InitrdMetadata {
    pub num_files: usize,
    pub total_files_size: usize,
}

pub struct InitrdFileEntry {
    pub name: String,
    pub size: usize,
    pub offset: usize,
}

pub struct InitrdData<'a> {
    pub metadata: InitrdMetadata,
    pub file_entries: Vec<InitrdFileEntry>,
    pub data: &'a [u8],
}

impl InitrdData<'_> {
    pub fn new(
        metadata: InitrdMetadata,
        file_entries: Vec<InitrdFileEntry>,
        data: &'static [u8],
    ) -> InitrdData<'static> {
        InitrdData {
            metadata,
            file_entries,
            data,
        }
    }
}

pub static INITRDDATA: OnceCell<Spinlock<InitrdData>> = OnceCell::uninit();

/// # Safety
/// Uses Raw Pointer To Make A Slice
pub unsafe fn init(ramdisk_mem: *const u8, ramdisk_len: u64) {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Initializing initrd"), file!(), line!());

    LOGGER.get().unwrap().lock().info("Initializing initrd");

    INITRDDATA.init_once(|| {
        let data = unsafe { get_data(ramdisk_mem, ramdisk_len) };
        let metadata = parse_initrd_metadata(data).unwrap();
        let file_entries = parse_initrd_file_entries(data).unwrap();
        spinning_top::Spinlock::new(InitrdData::new(metadata, file_entries, data))
    });
}

// # Safety
/// Uses Raw Pointer To Make A Slice
unsafe fn get_data(ramdisk_mem: *const u8, ramdisk_len: u64) -> &'static [u8] {
    unsafe { core::slice::from_raw_parts(ramdisk_mem, ramdisk_len.try_into().unwrap()) }
}

fn parse_initrd_metadata(data: &[u8]) -> Option<InitrdMetadata> {
    let num_files_key = b"Number Of Files:";
    let total_files_size_key = b"Total Files Size:";

    let num_files_value = extract_value(data, num_files_key, None)?;
    let total_files_size_value = extract_value(data, total_files_size_key, None)?;

    let num_files = core::str::from_utf8(num_files_value)
        .ok()?
        .trim()
        .parse()
        .ok()?;
    let total_files_size = core::str::from_utf8(total_files_size_value)
        .ok()?
        .trim()
        .parse()
        .ok()?;

    Some(InitrdMetadata {
        num_files,
        total_files_size,
    })
}

fn parse_initrd_file_entries(data: &[u8]) -> Option<Vec<InitrdFileEntry>> {
    let file_entry_start = b"File Entry:";
    let file_entry_end = b"File Entry End:";
    let file_entry_sections = extract_sections(data, file_entry_start, file_entry_end)?;

    let mut file_entries = Vec::new();

    for file_entry_section in file_entry_sections {
        let name_key = b"Name:";
        let size_key = b"Size:";
        let offset_key = b"Offset:";

        let name = extract_value(file_entry_section, name_key, Some(b"\n"))?;
        let size = extract_value(file_entry_section, size_key, Some(b"\n"))?;
        let offset = extract_value(file_entry_section, offset_key, Some(b"\n"))?;

        let name_str = core::str::from_utf8(name).ok()?.trim();
        let name_string = name_str.to_string();
        let size_value = core::str::from_utf8(size).ok()?.trim().parse().ok()?;
        let offset_value = core::str::from_utf8(offset).ok()?.trim().parse().ok()?;

        file_entries.push(InitrdFileEntry {
            name: name_string,
            size: size_value,
            offset: offset_value,
        });
    }

    Some(file_entries)
}

fn extract_section<'a>(data: &'a [u8], start_marker: &[u8], end_marker: &[u8]) -> Option<&'a [u8]> {
    let start_pos = data
        .windows(start_marker.len())
        .position(|window| window == start_marker)?;
    let end_pos = data
        .windows(end_marker.len())
        .position(|window| window == end_marker)?;

    Some(&data[start_pos + start_marker.len()..end_pos])
}

fn extract_sections<'a>(
    data: &'a [u8],
    start_marker: &'a [u8],
    end_marker: &'a [u8],
) -> Option<Vec<&'a [u8]>> {
    let mut sections = Vec::new();
    let mut start = 0;

    while let Some(start_pos) = data[start..]
        .windows(start_marker.len())
        .position(|window| window == start_marker)
    {
        let section_start = start + start_pos + start_marker.len();
        let end_pos = data[section_start..]
            .windows(end_marker.len())
            .position(|window| window == end_marker);

        if let Some(end_pos) = end_pos {
            let section_end = section_start + end_pos;
            sections.push(&data[section_start..section_end]);
            start = section_end + end_marker.len();
        } else {
            // No end marker found, invalid format
            return None;
        }
    }

    Some(sections)
}

fn extract_value<'a>(
    data: &'a [u8],
    key: &[u8],
    termination_marker: Option<&[u8]>,
) -> Option<&'a [u8]> {
    let key_pos = data.windows(key.len()).position(|window| window == key)?;
    let value_start = key_pos + key.len();

    let value_end = if let Some(termination_marker) = termination_marker {
        data[value_start..]
            .windows(termination_marker.len())
            .position(|window| window == termination_marker)?
    } else {
        data[value_start..].iter().position(|&byte| byte == b'\n')?
    };

    Some(&data[value_start..value_start + value_end])
}

fn extract_data_section(data: &[u8]) -> Option<&[u8]> {
    extract_section(data, b"Data:", b"Data End:")
}

pub fn get_file_names() -> Option<Vec<alloc::string::String>> {
    let initrddata = INITRDDATA.get();
    initrddata?;

    let mut file_names: Vec<String> = Vec::new();
    let file_entries = &initrddata.unwrap().lock().file_entries;
    for file_entry in file_entries {
        file_names.push(file_entry.name.clone());
    }
    Some(file_names)
}

pub fn get_file_contents(file_name: &str) -> Option<&str> {
    let initrddata = INITRDDATA.get();
    initrddata?;

    let raw_data = initrddata.unwrap().lock().data;
    let file_entries = &initrddata.unwrap().lock().file_entries;
    let data = extract_data_section(raw_data).unwrap();
    let lowercase_file_name = file_name.trim().to_lowercase();

    for file_entry in file_entries {
        if file_entry.name.trim().to_lowercase() == lowercase_file_name {
            let start_index = file_entry.offset;
            let end_index = start_index + file_entry.size;
            let file_data = &data[start_index..end_index];

            if let Ok(data) = core::str::from_utf8(file_data) {
                return Some(data);
            }
        }
    }

    None
}

pub fn number_of_files() -> Option<usize> {
    let initrddata = INITRDDATA.get();
    initrddata?;

    Some(initrddata.unwrap().lock().metadata.num_files)
}

pub fn total_files_size() -> Option<usize> {
    let initrddata = INITRDDATA.get();
    initrddata?;

    Some(initrddata.unwrap().lock().metadata.total_files_size)
}
