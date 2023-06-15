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

use std::io::{Write, BufWriter};
use std::path::{PathBuf, Path};
use std::fs::File;


fn main() {
    // set by cargo, build scripts should use this directory for output files
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    // set by cargo's artifact dependency feature
    let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());

    // create an UEFI disk image (optional)
    let uefi_path = out_dir.join("uefi.img");
    //bootloader::UefiBoot::new(&kernel).create_disk_image(&uefi_path).unwrap();
    // create a BIOS disk image
    let bios_path = out_dir.join("bios.img");

    //let file1: (&str, &[u8]) = ("file1.txt", b"Hello, World!");
    //let file2: (&str, &[u8]) = ("file2.txt", b"This is a test.");
    //let files: Vec<(&str, &[u8])> = vec![file1, file2];


    //generate_fat32_initrd(files);

    bootloader::BiosBoot::new(&kernel)
    // Add A Ram Disk File - Bootable Disk On Start
    //.set_ramdisk(Path::new("initrd.img"))
    .create_disk_image(&bios_path).unwrap();

    // pass the disk image paths as env variables to the `main.rs`
    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());

    let kernel_path = kernel.to_string_lossy();
    println!("cargo:rustc-env=KERNEL_PATH={}", kernel_path);

}


const SECTOR_SIZE: usize = 512;
const SECTORS_PER_CLUSTER: usize = 4;
const ROOT_ENTRIES: usize = 512;
const FAT_ENTRY_SIZE: usize = 4;
const BYTES_PER_CLUSTER: usize = SECTOR_SIZE * SECTORS_PER_CLUSTER;

fn generate_fat32_initrd(files: Vec<(&str, &[u8])>) {
    // Calculate required parameters
    let total_file_size: usize = files.iter().map(|(_, content)| content.len()).sum();
    let total_clusters = (total_file_size + BYTES_PER_CLUSTER - 1) / BYTES_PER_CLUSTER;
    let fat_size = ((total_clusters * FAT_ENTRY_SIZE) + SECTOR_SIZE - 1) / SECTOR_SIZE;
    let data_sectors = total_clusters * SECTORS_PER_CLUSTER;

    // Generate boot sector
    let mut boot_sector = vec![0u8; SECTOR_SIZE];

    // Add boot sector signature
    boot_sector[510] = 0x55;
    boot_sector[511] = 0xAA;

    // Generate FAT
    let mut fat = vec![0u8; fat_size * SECTOR_SIZE];

    // Initialize FAT entries
    let fat_entry_value = 0x0FFF_FFF8u32.to_le_bytes(); // Example FAT entry value
    for fat_entry in fat.chunks_exact_mut(4) {
        fat_entry.copy_from_slice(&fat_entry_value);
    }

    // Generate root directory
    let mut root_directory = vec![0u8; ROOT_ENTRIES * 32];
    
    // Create directory entries for each file
    let mut offset = 0;
    for &(file_name, file_content) in files.iter() {
        let mut directory_entry = vec![0u8; 32];
        let name_bytes = file_name.as_bytes();
        let truncated_name_bytes = &name_bytes[..11.min(name_bytes.len())];
    
        directory_entry[..truncated_name_bytes.len()].copy_from_slice(&truncated_name_bytes);
        directory_entry[11] = 0x20; // Attribute: Regular file
        directory_entry[28..32].copy_from_slice(&(file_content.len() as u32).to_le_bytes());
        root_directory[offset..offset + 32].copy_from_slice(&directory_entry);
        offset += 32;
    }

    // Generate data blocks
    let mut data_blocks = vec![0u8; data_sectors * SECTOR_SIZE];
    // Copy file contents to the data blocks
    let mut data_offset = 0;
    for (_, file_content) in files.iter() {
        if !file_content.is_empty() {
            data_blocks[data_offset..data_offset + file_content.len()].copy_from_slice(file_content);
        }
        data_offset += file_content.len();
    }
    

    // Combine all components into the final initrd
    let mut initrd = Vec::new();
    initrd.extend_from_slice(&boot_sector);
    initrd.extend_from_slice(&fat);
    initrd.extend_from_slice(&root_directory);
    initrd.extend_from_slice(&data_blocks);

    let output_path = "initrd.img";
    let file = File::create(output_path).unwrap();
    let mut writer = BufWriter::new(file);
    writer.write_all(&initrd).unwrap();
    writer.flush().unwrap();
    println!("Initrd written to {}", output_path);
}