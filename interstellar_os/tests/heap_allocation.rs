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
#![feature(custom_test_frameworks)]
#![test_runner(interstellar_os::test_runner)]
#![reexport_test_harness_main = "test_main"]

use interstellar_os as lib;

use alloc::boxed::Box;
use alloc::vec::Vec;
use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use lib::other::log::LOGGER;
use lib::serial_print;

extern crate alloc;

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

entry_point!(heap_allocation, config = &BOOTLOADER_CONFIG);

fn heap_allocation(boot_info: &'static mut BootInfo) -> ! {
    serial_print!("\nheap_allocation::heap_allocation...\t");
    lib::init(boot_info); // Start Interrupt Descriptor table ect.

    serial_print!("[Ok]\n");

    test_main();

    lib::exit_qemu(lib::QemuExitCode::Success);
}

//########################################
// Test Cases
//########################################

#[test_case]
fn simple_allocation() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Running simple allocation test", file!(), line!());
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

#[test_case]
fn large_vec() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Running large vec allocation test", file!(), line!());
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}

#[test_case]
fn many_boxes() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Running many boxes allocation test", file!(), line!());
    for i in 0..100000 {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}

#[test_case]
fn many_boxes_long_lived() {
    LOGGER.get().unwrap().lock().trace(
        "Running many long lived boxes allocation test",
        file!(),
        line!(),
    );
    let long_lived = Box::new(1);
    for i in 0..10000 {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1)
}
