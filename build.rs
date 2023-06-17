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

use std::path::{PathBuf, Path};
use std::fs::{self, File};
use std::io::{prelude::*, self};

fn main() {
    // set by cargo, build scripts should use this directory for output files
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    // set by cargo's artifact dependency feature
    let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());

    // Create initrd
    let _ = create_initrd();

    // create an UEFI disk image (optional)
    let uefi_path = out_dir.join("uefi.img");
    bootloader::UefiBoot::new(&kernel)
        .set_ramdisk(Path::new("initrd"))
        .create_disk_image(&uefi_path)
        .unwrap();
    
    

    // create a BIOS disk image
    let bios_path = out_dir.join("bios.img");
    bootloader::BiosBoot::new(&kernel)
        // Add A Ram Disk File - Bootable Disk On Start Thats Loaded Into Memory
        .set_ramdisk(Path::new("initrd"))
        .create_disk_image(&bios_path)
        .unwrap();

    // pass the disk image paths as env variables to the `main.rs`
    println!("cargo:rustc-env=UEFI_PATH={}", uefi_path.display());
    println!("cargo:rustc-env=BIOS_PATH={}", bios_path.display());

    let kernel_path = kernel.to_string_lossy();
    println!("cargo:rustc-env=KERNEL_PATH={}", kernel_path);
}


// Initrd Format

//Metadata:
//  File Entries Offset:
//  Number Of Files:
//  Total Files Size:
//Metadata End:
//File Entries:
//  Name:
//  Size:
//  Data Offset:



// Struct to represent a file entry
struct FileEntry {
    name: String,
    data: Vec<u8>,
    offset: usize,
}

fn create_initrd() -> io::Result<()> {
    let files = fs::read_dir("./initrd-files")?;
    let mut file_entries: Vec<FileEntry> = Vec::new();
    let mut total_file_size = 1;

    for entry in files.flatten() {

            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    let file_name = entry.file_name().into_string().unwrap();
                    let mut file = File::open(entry.path())?;
                    let mut data = Vec::new();
                    file.read_to_end(&mut data)?;
                    let file_entry = FileEntry {
                        name: file_name,
                        data: data.clone(),
                        offset: total_file_size,
                    };
                    total_file_size += data.len();
                    file_entries.push(file_entry);
                }
            }
    }

    let total_files = file_entries.len();

    let mut file = File::create("initrd")?;
    file.write_all("Metadata:\n".as_bytes())?;
    file.write_all(format!("Number Of Files: {}\n", total_files).as_bytes())?;
    file.write_all(format!("Total Files Size: {}\n", total_file_size).as_bytes())?;
    file.write_all("Metadata End:\n".as_bytes())?;

    for file_entry in &file_entries {
        file.write_all("File Entry:\n".as_bytes())?;
        file.write_all(format!("Name: {}\n", file_entry.name).as_bytes())?;
        file.write_all(format!("Size: {}\n", file_entry.data.len()).as_bytes())?;
        file.write_all(format!("Offset: {}\n", file_entry.offset).as_bytes())?;
        file.write_all("File Entry End:\n".as_bytes())?;
    }
    file.write_all("Data:\n".as_bytes())?;
    for file_entry in file_entries {
        file.write_all(&file_entry.data)?;
    }
    file.write_all("\nData End:".as_bytes())?;

    Ok(())
}