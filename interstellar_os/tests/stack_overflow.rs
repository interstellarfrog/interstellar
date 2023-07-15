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
#![feature(abi_x86_interrupt)]

use interstellar_os as lib;

use bootloader_api::{entry_point, BootInfo, BootloaderConfig};
use lazy_static::lazy_static;
use lib::other::log::{
    LogLevel, Logger, BACKTRACE, BACKTRACE_INDEX, LOGGER, MAX_BACKTRACE_ENTRIES,
};
use lib::{exit_qemu, serial_print, QemuExitCode};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

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

entry_point!(double_fault, config = &BOOTLOADER_CONFIG);

fn double_fault(_boot_info: &'static mut BootInfo) -> ! {
    serial_print!("\nstack_overflow::stack_overflow...\t");

    // Initialize the test logger
    LOGGER.init_once(|| {
        let screen_printing = false;
        let serial_printing = true;
        let log_level = LogLevel::Low;
        let tracing = true;

        spinning_top::Spinlock::new(Logger {
            screen_printing,
            serial_printing,
            log_level,
            tracing,
        })
    });

    BACKTRACE.init_once(|| spinning_top::Spinlock::new([None; MAX_BACKTRACE_ENTRIES]));

    BACKTRACE_INDEX.init_once(|| spinning_top::Spinlock::new(0));

    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Initialized logger"), file!(), line!());

    lib::gdt::init();

    init_test_idt();

    stack_overflow();

    panic!("Execution Continued After Stack Overflow")
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_print!("[Ok]");
    exit_qemu(QemuExitCode::Success);
}

lazy_static! {
    static ref TEST_IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        unsafe {
            idt.double_fault
                .set_handler_fn(test_double_fault_handler)
                .set_stack_index(lib::gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt
    };
}

pub fn init_test_idt() {
    TEST_IDT.load();
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    let mut i = 0;
    volatile::Volatile::new(&mut i).read(); // Stops The Recursion From Being Optimized
}
