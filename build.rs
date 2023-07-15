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

use std::fs::{self, File};
use std::io::{self, prelude::*};
use std::path::{Path, PathBuf};

use bootloader::BootConfig;
use bootloader_boot_config::LevelFilter;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=Cargo.lock");
    println!("cargo:rerun-if-changed=interstellar_os");
    println!("cargo:rerun-if-changed=initrd-files");
    println!("cargo:rerun-if-changed=test_runner");
    println!("cargo:rerun-if-changed=rust-toolchain");

    // set by cargo, build scripts should use this directory for output files
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    // set by cargo's artifact dependency feature
    let os_path =
        PathBuf::from(std::env::var_os("CARGO_BIN_FILE_INTERSTELLAR_OS_interstellar_os").unwrap());

    // Create initrd
    let _ = create_initrd();

    let mut boot_config = BootConfig::default();

    let profile = std::env::var("PROFILE").unwrap();

    if profile == "release" {
        boot_config.serial_logging = false;
        boot_config.frame_buffer_logging = false;
        boot_config.log_level = LevelFilter::Error;
    } else {
        boot_config.serial_logging = true;
        boot_config.frame_buffer_logging = true;
        boot_config.log_level = LevelFilter::Trace;
    }

    // create a BIOS disk image
    let bios_path = out_dir.join("bios.img");
    bootloader::BiosBoot::new(&os_path)
        // Add A initrd File - File On Start Thats Loaded Into Memory
        .set_ramdisk(Path::new("./target/initrd"))
        .set_boot_config(&boot_config)
        .create_disk_image(&bios_path)
        .unwrap();

    // create an UEFI disk image
    let uefi_path = out_dir.join("uefi.img");
    bootloader::UefiBoot::new(&os_path)
        // Add A initrd File - File On Start Thats Loaded Into Memory
        .set_ramdisk(Path::new("./target/initrd"))
        .set_boot_config(&boot_config)
        .create_disk_image(&uefi_path)
        .unwrap();

    let new_uefi_path = PathBuf::from("./target/interstellar_uefi.img");
    let new_bios_path = PathBuf::from("./target/interstellar_bios.img");
    let new_os_path = PathBuf::from("./target/interstellar_os");

    // Read environment variables that were set in the build script
    fs::copy(format!("{}", uefi_path.display()), new_uefi_path.clone()).unwrap();
    fs::copy(format!("{}", bios_path.display()), new_bios_path.clone()).unwrap();
    fs::copy(format!("{}", os_path.display()), new_os_path.clone()).unwrap();

    // pass the disk image paths as env variables to the `main.rs`
    println!("cargo:rustc-env=UEFI_PATH={}", new_uefi_path.display());
    println!("cargo:rustc-env=BIOS_PATH={}", new_bios_path.display());
    println!("cargo:rustc-env=OS_PATH={}", new_os_path.display());

    println!("OS path: {}", new_os_path.display());
    println!("BIOS OS path: {}", new_bios_path.display());
    println!("UEFI OS location: {}", new_uefi_path.display());
}

// Initrd Format

//Metadata:
//Number Of Files:
//Total Files Size:
//Metadata End:
//
//File Entry:     // For each file
//Name:
//Size:
//Data Offset:
//
//Data:           // Contains all the raw data
//Data End:

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

    let mut file = File::create("./target/initrd")?;
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
