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

use core::mem::size_of;
use core::slice;
use core::str;
use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use crate::println;

const BYTES_PER_SECTOR: usize = 512;
const SECTOR_SIGNATURE_OFFSET: usize = 510;
const SECTOR_SIGNATURE: [u8; 2] = [0x55, 0xAA];
const FAT32_PARTITION_ENTRY_OFFSET: usize = 0x1BE;
const FAT32_SIGNATURE: [u8; 8] = [0x46, 0x41, 0x54, 0x33, 0x32, 0x20, 0x20, 0x20];

#[repr(C, packed)]
struct PartitionEntry {
    status: u8,
    start_head: u8,
    start_sector_cylinder: [u8; 3],
    partition_type: u8,
    end_head: u8,
    end_sector_cylinder: [u8; 3],
    start_lba: u32,
    total_sectors: u32,
}

#[derive(Clone, Copy)]
#[repr(C, packed)]
struct BootSector {
    jmp_boot: [u8; 3],
    oem_name: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    num_fats: u8,
    max_root_entries: u16,
    total_logical_sectors: u32,
    media_descriptor: u8,
    sectors_per_fat: u32,
    sectors_per_track: u16,
    num_heads: u16,
    hidden_sectors: u32,
    total_logical_sectors_large: u32,
    physical_drive_number: u8,
    current_head: u8,
    boot_signature: u8,
    volume_id: [u8; 4],
    volume_label: [u8; 11],
    fs_type: [u8; 8],
    boot_code: [u8; 420],
    bootable_partition_signature: u16,
}

pub struct Fat32Reader {
    boot_sector: BootSector,
    fat_start_lba: u32,
    root_directory_start_cluster: u32,
    data_start_lba: u32,
    bytes_per_cluster: usize,
    file_allocation_table: &'static [u8],
    initrd: &'static [u8],
}

impl Fat32Reader {
    pub fn new(initrd: &'static [u8]) -> Option<Self> {
        let boot_sector: &BootSector = unsafe { &*(initrd[0..512].as_ptr() as *const BootSector) };
        
        let partition_signature = boot_sector.bootable_partition_signature;

        if partition_signature != u16::from_le_bytes(SECTOR_SIGNATURE) {
            println!("Invalid Bootsector Signature: found 0x{:04X}, expected 0x{:04X}", partition_signature, u16::from_le_bytes(SECTOR_SIGNATURE));
            return None; // Invalid bootable partition signature
        }

        if &boot_sector.fs_type != &FAT32_SIGNATURE {
            println!("Invalid Filesystem Signature");
            return None; // Invalid file system signature
        }

        let fat_start_lba = boot_sector.reserved_sectors as u32;
        let root_directory_start_cluster = (fat_start_lba + boot_sector.num_fats as u32 * boot_sector.sectors_per_fat as u32) * (boot_sector.bytes_per_sector as u32 / 4);
        let data_start_lba = root_directory_start_cluster + (boot_sector.max_root_entries as u32 * 32) / boot_sector.bytes_per_sector as u32;

        let bytes_per_cluster = boot_sector.bytes_per_sector as usize * boot_sector.sectors_per_cluster as usize;

        let fat_offset = fat_start_lba as usize * boot_sector.bytes_per_sector as usize;
        let fat_size = boot_sector.sectors_per_fat as usize * boot_sector.bytes_per_sector as usize;
        let fat_slice = &initrd[fat_offset..fat_offset + fat_size];

        Some(Self {
            boot_sector: *boot_sector,
            fat_start_lba,
            root_directory_start_cluster,
            data_start_lba,
            bytes_per_cluster,
            file_allocation_table: fat_slice,
            initrd,
        })
    }

    pub fn get_file_names(&self) -> Vec<String> {
        let mut file_names = Vec::new();

        let mut cluster = self.root_directory_start_cluster;

        loop {
            let cluster_offset = self.cluster_to_lba(cluster) as usize * self.bytes_per_cluster;
            let dir_slice = &self.initrd[cluster_offset..cluster_offset + self.bytes_per_cluster];

            let dir_entries = dir_slice.len() / size_of::<DirectoryEntry>();
            let dir_entries_ptr = dir_slice.as_ptr() as *const DirectoryEntry;
            let entries = unsafe { slice::from_raw_parts(dir_entries_ptr, dir_entries) };

            for entry in entries {
                if entry.is_end_of_directory() {
                    return file_names;
                }

                if entry.is_deleted() {
                    continue; // Skip deleted entries
                }

                if entry.is_directory() {
                    continue; // Skip directories
                }

                if entry.has_long_file_name() {
                    continue; // Skip long file name entries
                }

                let entry_file_name = entry.get_file_name();
                let file_name = str::from_utf8(entry_file_name).unwrap().trim_end_matches(' ').to_owned();
                file_names.push(file_name);
            }

            cluster = self.get_next_cluster(cluster).unwrap();
        }
    }

    pub fn read_file(&self, file_path: &str) -> Option<&'static [u8]> {
        let mut cluster = self.root_directory_start_cluster;
        let file_name = Self::format_file_name(file_path);
        let file_name_bytes = file_name.as_bytes();

        loop {
            let cluster_offset = self.cluster_to_lba(cluster) as usize * self.bytes_per_cluster;
            let dir_slice = &self.initrd[cluster_offset..cluster_offset + self.bytes_per_cluster];

            let dir_entries = dir_slice.len() / size_of::<DirectoryEntry>();
            let dir_entries_ptr = dir_slice.as_ptr() as *const DirectoryEntry;
            let entries = unsafe { slice::from_raw_parts(dir_entries_ptr, dir_entries) };

            for entry in entries {
                if entry.is_end_of_directory() {
                    return None; // File not found
                }

                if entry.is_deleted() {
                    continue; // Skip deleted entries
                }

                if entry.is_directory() {
                    continue; // Skip directories
                }

                if entry.has_long_file_name() {
                    continue; // Skip long file name entries
                }

                let entry_file_name = entry.get_file_name();
                if entry_file_name == file_name_bytes {
                    let file_start_lba = self.cluster_to_lba(entry.starting_cluster) as usize * self.bytes_per_cluster;
                    let file_size = entry.file_size as usize;

                    return Some(&self.initrd[file_start_lba..file_start_lba + file_size]);
                }
            }

            cluster = self.get_next_cluster(cluster)?;
        }
    }

    fn cluster_to_lba(&self, cluster: u32) -> u32 {
        self.data_start_lba + (cluster - 2) * (self.boot_sector.sectors_per_cluster as u32)
    }

    fn get_next_cluster(&self, current_cluster: u32) -> Option<u32> {
        let fat_entry_offset = current_cluster as usize * 4;
        let fat_entry = u32::from_le_bytes([
            self.file_allocation_table[fat_entry_offset],
            self.file_allocation_table[fat_entry_offset + 1],
            self.file_allocation_table[fat_entry_offset + 2],
            self.file_allocation_table[fat_entry_offset + 3],
        ]);

        if fat_entry >= 0x0FFFFFF8 {
            None // End of file
        } else {
            Some(fat_entry)
        }
    }

    fn format_file_name(file_path: &str) -> String {
        let mut formatted_name = String::with_capacity(11);
        let mut chars_written = 0;

        for c in file_path.chars() {
            if c == '.' {
                chars_written = 8; // Start writing the extension
            } else {
                if chars_written < 11 {
                    formatted_name.push(c.to_ascii_uppercase());
                    chars_written += 1;
                }
            }
        }

        while chars_written < 11 {
            formatted_name.push(' ');
            chars_written += 1;
        }

        formatted_name
    }
}

#[repr(C, packed)]
struct DirectoryEntry {
    name: [u8; 11],
    attributes: u8,
    _reserved: [u8; 8],
    starting_cluster: u32,
    file_size: u32,
}

impl DirectoryEntry {
    fn is_end_of_directory(&self) -> bool {
        self.name[0] == 0x00
    }

    fn is_deleted(&self) -> bool {
        self.name[0] == 0xE5
    }

    fn is_directory(&self) -> bool {
        self.attributes & 0x10 != 0
    }

    fn has_long_file_name(&self) -> bool {
        self.attributes == 0x0F
    }

    fn get_file_name(&self) -> &[u8] {
        &self.name[..]
    }
}