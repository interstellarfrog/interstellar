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

#![no_std]
#![no_main]
#![feature(custom_test_frameworks)] // Allows Us To Run Custom Tests
#![test_runner(interstellar_os::test_runner)] // Defines The Test Runner Function
#![reexport_test_harness_main = "test_main"]

use core::time::Duration;

use interstellar_os as lib;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use lib::{other::log::LOGGER, serial_print};

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
    config.kernel_stack_size = 48 * 1024; // 48 Kib   decreasing this will cause undefined behavior
    config
};

entry_point!(time, config = &BOOTLOADER_CONFIG);

fn time(boot_info: &'static mut BootInfo) -> ! {
    serial_print!("\ntime::time...\t");
    lib::init(boot_info); // Start Interrupt Descriptor table ect.

    serial_print!("[Ok]\n");

    test_main();

    lib::exit_qemu(lib::QemuExitCode::Success);
}

//########################################
// Test Cases
//########################################

#[test_case]
fn sleep() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Running sleep test", file!(), line!());

    let timer = lib::time::Timer::new();

    // The LAPIC timer ticks every 10ms
    // So every 100 ticks is 1 second

    let start_count = unsafe { lib::time::APIC_COUNT.load(core::sync::atomic::Ordering::SeqCst) };

    timer.sleep(Duration::from_secs(1));

    let end_count = unsafe { lib::time::APIC_COUNT.load(core::sync::atomic::Ordering::SeqCst) };

    // So if this is close to 100 we know the timings are semi correct

    let slept_for = end_count - start_count;

    if slept_for > 130 || slept_for < 70 {
        panic!("Timing should be close to 100 but it is {}", slept_for)
    }
}

#[test_case]
fn elapsed() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Running sleep test", file!(), line!());

    let timer = lib::time::Timer::new();

    timer.sleep(Duration::from_secs(1));

    let elapsed = timer.elapsed().as_secs_f64();

    if elapsed > 1.3 || elapsed < 0.7 {
        panic!("Timing should be close to 1 but was {}", elapsed)
    }
}
