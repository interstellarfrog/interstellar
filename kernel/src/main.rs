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

#![no_std] // We Cannot Use The Standard Lib As It has OS Specific Functions.
#![no_main]
// A Function Is Called Before The Main Function Which Sets Up The Environment So We Need To OverWrite This As We Do Not Have The OS We Are Coding One
#![feature(custom_test_frameworks)] // Allows Us To Run Custom Tests
#![test_runner(kernel::test_runner)] // Defines The Test Runner Function
#![reexport_test_harness_main = "test_main"]
// No Main Makes This Not Run As Behind The Scenes Main Is Called For Testing - So We Change The Name
#![allow(unreachable_code)]

use core::panic::PanicInfo;

use kernel::{
    drivers::{screen::framebuffer::FRAMEBUFFER, fs::initrd::{parse_initrd_file_entries, get_file_contents, get_file_names}},
    serial_println,
    task::{console_handler::console_start, executor::Spawner},
};

use bootloader_api::{
    config::{BootloaderConfig, Mapping},
    entry_point, BootInfo,
};
use kernel::assembly::hlt_loop;
use kernel::task::executor::Executor;
use kernel::drivers::fs::initrd::parse_initrd_metadata;

// Used To Call Real Debug Println If In Debug Mode
// If Not In Debug Mode It Should Be Optimized Away By The Compiler

use kernel::debug_println;

#[cfg(debug_assertions)]
use kernel::{
    debug::{set_debug_mode, DEBUG_LOCK, DEBUG_MODE},
    real_debug_println,
};

use kernel::println;

extern crate alloc;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

#[no_mangle]
/// Start Point Of The Operating System
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    #[cfg(debug_assertions)]
    set_debug_mode(true);

    let mut ramdisk_location = 0;
    let mut ramdisk_len = 0;

    if boot_info.ramdisk_addr.as_ref().is_some() {
        ramdisk_location = *boot_info.ramdisk_addr.as_ref().unwrap();
        ramdisk_len = boot_info.ramdisk_len;
    }
    kernel::init(boot_info); // Start Interrupt Descriptor table ect.
    let buffer_info = FRAMEBUFFER.get().as_mut().unwrap().lock().buffer_info();
    debug_println!(
        "Screen Resolution {}x{}\n",
        buffer_info.width,
        buffer_info.height
    );
    debug_println!("Initial Ramdisk Location: {}", ramdisk_location);
    debug_println!("Initial Ramdisk Size: {}", ramdisk_len);
    if ramdisk_len > 0 {
    let ramdisk_mem = ramdisk_location as *const u8;
    let initrd_data = unsafe{core::slice::from_raw_parts(ramdisk_mem, ramdisk_len as usize)};
    let metadata = parse_initrd_metadata(initrd_data);
    

    if let Some(metadata) = metadata {
        debug_println!("Number of files: {}", metadata.num_files);
        debug_println!("Total files size: {}", metadata.total_files_size);
    }
    let file_entries = parse_initrd_file_entries(initrd_data).unwrap();
    let file_names = get_file_names(&file_entries);
    debug_println!("File Names: {:?}", file_names);
    let test1 = get_file_contents(&file_entries, initrd_data, "test.txt").unwrap();
    let test2 = get_file_contents(&file_entries, initrd_data, "test2.txt").unwrap();

    debug_println!("test1: {}", test1);
    debug_println!("test2: {}", test2);

    }

    kernel::drivers::hid::mouse::init();


    debug_println!("Creating Task Executor");

    let spawner = Spawner::new(100);
    let mut executor = Executor::new(spawner.clone());

    spawner.add(console_start());
    spawner.add(kernel::task::mouse::process());

    executor.run();

    #[cfg(test)]
    test_main();

    #[allow(clippy::empty_loop)]
    hlt_loop(); // Loop Until Next Interrupt - Saves CPU Percentage
}

#[cfg(not(test))] // If Not In Test
#[panic_handler]
// This function is called on panic.
fn panic(info: &PanicInfo) -> ! {
    // The Panic Info Contains Information About The Panic.
    serial_println!("\nError: {}", info);
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
// This function is called on panic.
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info)
}
