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
use lib::{drivers::random::RandomNumberGenerator, other::log::LOGGER, serial_print};

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

entry_point!(random, config = &BOOTLOADER_CONFIG);

fn random(boot_info: &'static mut BootInfo) -> ! {
    serial_print!("\nrandom::random...\t");
    lib::init(boot_info); // Start Interrupt Descriptor table ect.

    serial_print!("[Ok]\n");

    test_main();

    lib::exit_qemu(lib::QemuExitCode::Success);
}

//########################################
// Test Cases
//########################################

#[test_case]
fn random_numbers_0_to_100() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Running random numbers 0 to 100 test", file!(), line!());
    let mut rng = RandomNumberGenerator::default();

    for _ in 0..50 {
        let number = rng.generate_number(Some(0), Some(100));
        if number.is_none() {
            panic!("Random number generator gave a 'None' value");
        }
    }
}

#[test_case]
fn random_numbers_0_to_max() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Running random numbers 0 to max test", file!(), line!());
    let mut rng = RandomNumberGenerator::default();

    for _ in 0..50 {
        let number = rng.generate_number(Some(0), None);
        if number.is_none() {
            panic!("Random number generator gave a 'None' value");
        }
    }
}

#[test_case]
fn random_numbers_min_to_2_147_483_646() {
    LOGGER.get().unwrap().lock().trace(
        "Running random numbers min to 2,147,483,646 test",
        file!(),
        line!(),
    );
    let mut rng = RandomNumberGenerator::default();

    for _ in 0..50 {
        let number = rng.generate_number(None, Some(2147483646));
        if number.is_none() {
            panic!("Random number generator gave a 'None' value");
        }
    }
}

#[test_case]
fn random_letters() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Running random letters test", file!(), line!());
    let mut rng = RandomNumberGenerator::default();

    for _ in 0..50 {
        let letter = rng.generate_letter();
        if letter.is_none() {
            panic!("Random letter generator gave a 'None' value");
        }
    }
}
