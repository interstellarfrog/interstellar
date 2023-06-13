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
#![cfg_attr(test, no_main)] // If test - No Main
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]

#[cfg(test)]
use bootloader_api::entry_point;
use drivers::screen::framebuffer::{FRAMEBUFFER, FrameBufferWriter};

#[cfg(test)]
entry_point!(test_kernel_main);

use core::panic::PanicInfo;
use x86_64::{instructions::{port::Port}, VirtAddr};
use assembly::hlt_loop;
use bootloader_api::BootInfo;

#[cfg(debug_assertions)]
use crate::debug::DEBUG_MODE;
#[cfg(debug_assertions)]
use crate::debug::DEBUG_LOCK;

pub mod drivers {
    pub mod hid {
        pub mod mouse;
    }
    pub mod screen {
        pub mod framebuffer;
    }
}

pub mod serial;
pub mod interrupts;
pub mod syscall;
pub mod gdt;
pub mod memory;
pub mod allocator;
pub mod assembly;
pub mod task;
pub mod console;
pub mod tests;

pub mod debug;

extern crate alloc;

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
fn test_kernel_main(boot_info: &'static mut BootInfo) -> ! {
    init(boot_info);
    test_main();
    hlt_loop(); // Loop Until Next Interrupt - Saves CPU Percentage
}

/// Initializes the kernel by setting up the framebuffer, creating the kernel heap, initializing the Global Descriptor Table (GDT),
/// Interrupt Descriptor Table (IDT), Programmable Interrupt Controllers (PICs), enabling interrupts, and performing other necessary initialization steps.
pub fn init(boot_info: &'static mut BootInfo) {
    // Initialize the framebuffer
    FRAMEBUFFER.init_once(|| {
        let frame = boot_info.framebuffer.as_mut();
        let info = match frame {
            Some(ref v) => v.info(),
            None => panic!("BOOTLOADER NOT CONFIGURED TO SUPPORT FRAMEBUFFER"),
        };
        let buffer = match frame {
            Some(v) => v.buffer_mut(),
            None => panic!("BOOTLOADER NOT CONFIGURED TO SUPPORT FRAMEBUFFER"),
        };
        spinning_top::Spinlock::new(FrameBufferWriter::new(buffer, info))
    });

    // Initialize other components
    debug_println!("[Kernel Init]\n");

    let phys_mem_offset = VirtAddr::new(*boot_info.physical_memory_offset.as_ref().unwrap());   
    let mut mapper = unsafe { memory::init(phys_mem_offset)};
    let mut frame_allocator = unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    debug_println!("Creating Kernel Heap...");

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    debug_println!("Kernel Heap Created");

    debug_println!("GDT Init...");

    gdt::init();

    debug_println!("GDT Init Finished");

    debug_println!("IDT Init...");

    interrupts::init_idt();

    debug_println!("IDT Init Finished");

    debug_println!("PIC Init...");

    unsafe { interrupts::PICS.lock().initialize() }; // Init Hardware Interrupt Controllers

    debug_println!("PIC Init Finished");

    debug_println!("Enabling Interrupts...");

    x86_64::instructions::interrupts::enable(); // Enable Hardware Interrupts 

    debug_println!("Interrupts Enabled");

    debug_println!("[Kernel Init Finished]");
}

/// Trait for testable functions
pub trait Testable {
    /// Runs the test
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>()); // Print Function Name
        self(); // Call The Test
        serial_println!("[ok]"); // If No Panic
    }
}

/// Runs a collection of tests
pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

/// Panic handler for tests
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop(); // Loop Until Next Interrupt - Saves CPU Percentage
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}

/// Exit codes for QEMU emulator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// Exits QEMU emulator with the specified exit code
pub fn exit_qemu(exit_code: QemuExitCode) {
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32)
    }
}

/// Allocation error handler
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
