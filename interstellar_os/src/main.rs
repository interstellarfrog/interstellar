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

#![no_std] // We Cannot Use The Standard Lib As It has OS Specific Functions.
#![no_main] // Required for bootloader lets us redefine the main function

extern crate alloc;

use interstellar_os as lib;

use alloc::format;
use bootloader_api::{config::BootloaderConfig, entry_point, BootInfo};

use lib::{
    drivers::{
        fs::initrd::{get_file_names, number_of_files, total_files_size},
        screen::framebuffer::FRAMEBUFFER,
    },
    other::{
        info::{BOOT_INFO, INFO},
        log::LOGGER,
    },
    println,
};

use lib::task::{console_handler::console_start, executor::Executor, executor::Spawner};

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    use bootloader_api::config::*;

    let mut mappings = Mappings::new_default();
    //mappings.kernel_stack = Mapping::Dynamic;
    //mappings.boot_info = Mapping::Dynamic;
    //mappings.framebuffer = Mapping::Dynamic;
    mappings.physical_memory = Some(Mapping::Dynamic); // This is very important do not disable

    //mappings.page_table_recursive = None;
    //mappings.aslr = true;
    //mappings.dynamic_range_start = Some(0xFFFF_8000_0000_0000);
    //mappings.dynamic_range_end = Some(0xFFFF_FFFF_FFFF_FFFF);

    let mut config = BootloaderConfig::new_default();
    config.mappings = mappings;
    config.kernel_stack_size = 48 * 1024; // 48 Kib   decreasing this will cause undefined behavior
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

/// Start Point Of The Operating System
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    lib::init(boot_info); // Start Interrupt Descriptor table ect.

    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Getting startup info", file!(), line!());

    let buffer_info = FRAMEBUFFER.get().as_mut().unwrap().lock().buffer_info();
    let number_of_files = number_of_files().unwrap_or(0);
    let total_files_size = total_files_size().unwrap_or(0);
    let file_names = get_file_names().unwrap_or_default();

    let version_major = BOOT_INFO.get().unwrap().lock().api_version.version_major();
    let version_minor = BOOT_INFO.get().unwrap().lock().api_version.version_minor();
    let version_patch = BOOT_INFO.get().unwrap().lock().api_version.version_patch();

    let interstellar_os_version = INFO.get().unwrap().lock().os_version;
    let bootloader_api_version = format!("{}.{}.{}", version_major, version_minor, version_patch);

    println!("Interstellar OS Version {}", interstellar_os_version);
    println!("Bootloader Version: {}\n", bootloader_api_version.as_str());

    LOGGER.get().unwrap().lock().info(
        format!(
            "Screen Resolution {}x{}",
            buffer_info.width, buffer_info.height
        )
        .as_str(),
    );
    LOGGER
        .get()
        .unwrap()
        .lock()
        .info(format!("Number of initrd files: {}", number_of_files).as_str());
    LOGGER
        .get()
        .unwrap()
        .lock()
        .info(format!("Total files size: {}", total_files_size).as_str());
    LOGGER
        .get()
        .unwrap()
        .lock()
        .info(format!("File Names: {:?}", file_names).as_str());

    LOGGER.get().unwrap().lock().info("Creating Task Executor");
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Creating Task Executor", file!(), line!());

    let spawner = Spawner::new(100);

    let mut executor = Executor::new(spawner.clone());

    spawner.add(console_start());

    spawner.add(lib::task::mouse::process());

    executor.run();
}
