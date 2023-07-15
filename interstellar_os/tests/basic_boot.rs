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

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)] // Allows Us To Run Custom Tests
#![test_runner(interstellar_os::test_runner)] // Defines The Test Runner Function
#![reexport_test_harness_main = "test_main"]

use interstellar_os as lib;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use lib::{other::log::LOGGER, serial_print};

#[cfg(feature = "UEFI")]
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    use bootloader_api::config::*;

    let mut mappings = Mappings::new_default();
    mappings.kernel_stack = Mapping::Dynamic;
    mappings.boot_info = Mapping::Dynamic;
    mappings.framebuffer = Mapping::Dynamic;
    mappings.physical_memory = Some(Mapping::Dynamic);
    mappings.page_table_recursive = None;
    mappings.aslr = true;
    mappings.dynamic_range_start = Some(0xFFFF_8000_0000_0000);
    mappings.dynamic_range_end = Some(0xFFFF_FFFF_FFFF_FFFF);

    let mut config = BootloaderConfig::new_default();
    config.mappings = mappings;
    config.kernel_stack_size = 80 * 1024 * 128;
    config
};

#[cfg(not(feature = "UEFI"))]
pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    use bootloader_api::config::*;

    let mut mappings = Mappings::new_default();
    mappings.kernel_stack = Mapping::Dynamic;
    mappings.boot_info = Mapping::Dynamic;
    mappings.framebuffer = Mapping::Dynamic;
    mappings.physical_memory = Some(Mapping::Dynamic);
    mappings.page_table_recursive = None;
    mappings.aslr = false;
    mappings.dynamic_range_start = Some(0xFFFF_8000_0000_0000);
    mappings.dynamic_range_end = Some(0xFFFF_FFFF_FFFF_FFFF);

    let mut config = BootloaderConfig::new_default();
    config.mappings = mappings;
    config.kernel_stack_size = 80 * 1024 * 128;
    config
};

entry_point!(basic_boot, config = &BOOTLOADER_CONFIG);

fn basic_boot(boot_info: &'static mut BootInfo) -> ! {
    serial_print!("\nbasic_boot::basic_boot...\t");
    lib::init(boot_info); // Start Interrupt Descriptor table ect.

    serial_print!("[Ok]\n");

    test_main();

    lib::exit_qemu(lib::QemuExitCode::Success);
}

//########################################
// Test Cases
//########################################

#[test_case]
fn simple_addition() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Running simple addition"), file!(), line!());
    assert_eq!(1 + 1, 1 + 1);
}

#[test_case]
fn simple_subtraction() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Running simple subtraction"), file!(), line!());
    assert_eq!(5 - 2, 3);
}

#[test_case]
fn simple_multiplication() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Running simple multiplication"), file!(), line!());
    assert_eq!(5 * 5, 5 * 5);
}

#[test_case]
fn simple_division() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Running simple division"), file!(), line!());
    assert_eq!(50 / 5, 50 / 5);
}
