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
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]
#![feature(let_chains)]
#![cfg(not(feature = "std"))]

use bootloader_api::BootInfo as BI;
use drivers::fs::initrd;
use drivers::screen::framebuffer::{FrameBufferWriter, FRAMEBUFFER};
use other::assembly::hlt_loop;
use other::info::BOOT_INFO;
use other::log::{self, LogLevel, LOGGER};
use x86_64::instructions::port::Port;
use x86_64::PhysAddr;

pub mod drivers {
    pub mod hid {
        pub mod keyboard;
        pub mod mouse;
    }
    pub mod screen {
        pub mod framebuffer;
    }
    pub mod fs {
        pub mod initrd;
    }
    pub mod io {
        pub mod serial;
    }
    pub mod random;
    //pub mod gui;
}

pub mod acpi;
pub mod allocator;
pub mod interrupts;
pub mod memory;
pub mod time;

pub mod other {
    pub mod assembly;
    pub mod console;
    pub mod info;
    pub mod log;
    pub mod tests;
}

pub mod gdt;
pub mod syscall;
pub mod task;

extern crate alloc;

/// Initializes the kernel by setting up the framebuffer, creating the kernel heap, initializing the Global Descriptor Table (GDT),
/// Interrupt Descriptor Table (IDT), Advanced Programmable Interrupt Controllers (APICs), enabling interrupts, and performing other necessary initialization
pub fn init(boot_info: &'static mut BI) {
    // Initialize the logger
    #[cfg(not(debug_assertions))]
    #[cfg(not(feature = "test"))]
    log::init(true, false, LogLevel::Normal, true);

    // Initialize the debug logger
    #[cfg(debug_assertions)]
    #[cfg(not(feature = "test"))]
    log::init(true, true, LogLevel::High, true);

    // Initialize the test logger
    #[cfg(feature = "test")]
    log::init(false, true, LogLevel::Low, true);

    //##################################
    //    Initialize And Test Info
    //##################################

    // Required

    let framebuffer_info; // Needed to write to screen
    let physical_memory_offset; // Needed to translate virt addr to phys addr
    let rsdp_address; // Needed to Parse ACPI Tables

    // Not Required

    let mut ramdisk_address = None; // Will Be Required In Future For Drivers Ect
    let mut recursive_index = None; // Never Needed And Never Will Be Used
    let mut tls_template = None; // Dont Know If We Need This

    // Needed to write to screen
    if let Some(fb) = boot_info.framebuffer.as_mut() {
        framebuffer_info = fb.info();
    } else {
        LOGGER.get().unwrap().lock().trace(
            "Bootloader Has Not Passed Framebuffer",
            file!(),
            line!(),
        );
        panic!("Bootloader Has Not Passed Framebuffer");
    }

    // Needed to translate virt addr to phys addr
    if let Some(phys_mem_offset) = boot_info.physical_memory_offset.as_mut().cloned() {
        physical_memory_offset = phys_mem_offset;
    } else {
        LOGGER.get().unwrap().lock().trace(
            "Bootloader Has Not Passed Physical Memory Offset",
            file!(),
            line!(),
        );
        panic!("Bootloader Has Not Passed Physical Memory Offset");
    }

    // Needed to Parse ACPI Tables
    if let Some(rsdp_addr) = boot_info.rsdp_addr.as_mut().cloned() {
        rsdp_address = rsdp_addr;
    } else {
        LOGGER
            .get()
            .unwrap()
            .lock()
            .trace("RSDP Address Not Found", file!(), line!());
        panic!("RSDP Address Not Passed To Kernel");
    }

    // Will Be Required In Future For Drivers Ect
    if let Some(ramdisk_addr) = boot_info.ramdisk_addr.as_mut().cloned() {
        ramdisk_address = Some(ramdisk_addr);
    } else {
        LOGGER
            .get()
            .unwrap()
            .lock()
            .trace("Ramdisk Addr Not Found", file!(), line!());
    }

    // Never Needed And Never Will Be Used
    if let Some(recurs_index) = boot_info.recursive_index.as_mut().cloned() {
        recursive_index = Some(recurs_index);
    } else {
        LOGGER
            .get()
            .unwrap()
            .lock()
            .trace("Recursive Index Not Found", file!(), line!());
    }

    // Dont Know If We Need This
    if let Some(tls_templ) = boot_info.tls_template.as_mut().cloned() {
        tls_template = Some(tls_templ);
    } else {
        LOGGER
            .get()
            .unwrap()
            .lock()
            .trace("TLS Template Not Found", file!(), line!());
    }

    // Initialize Kernel Info
    crate::other::info::boot_info_init(
        boot_info.api_version,
        framebuffer_info,
        physical_memory_offset,
        recursive_index,
        rsdp_address,
        tls_template,
        ramdisk_address,
        boot_info.ramdisk_len,
    );
    crate::other::info::info_init();

    // This Is Just Initializing The Struct That Holds The Console lines
    crate::task::console_handler::init();

    // Initialize Framebuffer
    {
        LOGGER
            .get()
            .unwrap()
            .lock()
            .trace("Initializing framebuffer", file!(), line!());

        FRAMEBUFFER.init_once(|| {
            spinning_top::Spinlock::new(FrameBufferWriter::new(
                boot_info.framebuffer.as_mut().unwrap().buffer_mut(),
                BOOT_INFO.get().unwrap().lock().framebuffer_info,
            ))
        });
    }

    // Initialize Memory And Heap
    {
        LOGGER
            .get()
            .unwrap()
            .lock()
            .trace("Initializing Memory", file!(), line!());

        allocator::init(
            Some(BOOT_INFO.get().unwrap().lock().physical_memory_offset),
            &mut boot_info.memory_regions,
        );
    }

    // Do not use print, println before this point

    if ramdisk_address.is_none() {
        LOGGER
            .get()
            .unwrap()
            .lock()
            .warn("Ramdisk Address Has Not Been Passed To Kernel");
    }

    if tls_template.is_none() {
        LOGGER.get().unwrap().lock().warn("TLS Template Not Found");
    }

    LOGGER.get().unwrap().lock().info("Logger initialized");

    LOGGER.get().unwrap().lock().info("Framebuffer initialized");

    LOGGER.get().unwrap().lock().info("Memory initialized");

    LOGGER.get().unwrap().lock().info(&alloc::format!(
        "total memory: {}",
        memory::MEMORY.get().unwrap().lock().total_memory
    ));
    LOGGER.get().unwrap().lock().info(&alloc::format!(
        "total memory: {}GB",
        memory::MEMORY.get().unwrap().lock().total_mem_gigabytes()
    ));

    // Initialize The INITRD
    {
        let ramdisk_addr = BOOT_INFO.get().unwrap().lock().ramdisk_addr; // Stops deadlock

        if let Some(ramdisk_addr) = ramdisk_addr {
            let ramdisk_len = BOOT_INFO.get().unwrap().lock().ramdisk_len;
            unsafe { initrd::init(ramdisk_addr as *const u8, ramdisk_len) };
        } else {
            LOGGER.get().unwrap().lock().info("No initrd found");
        }
    }

    // Initialize The Global Descriptor Table
    gdt::init();

    // Initialize The Time Struct
    time::init();

    // Parse The ACPI Tables
    let acpi_tables = acpi::init(PhysAddr::new(boot_info.rsdp_addr.into_option().unwrap()));

    // Create IDT And APIC Structures And Enable Interrupts
    interrupts::init(&acpi_tables.platform_info().unwrap());

    // Enable The PS/2 Keyboard
    drivers::hid::keyboard::init();

    // Enable The PS/2 Mouse
    drivers::hid::mouse::init();

    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Initialized kernel", file!(), line!());
}

/// Allocation error handler
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

// All below false errors are mostly for VScode as rust analyzer runs commands like cargo check --workspace or cargo clippy --workspace.
// I recommend changing your "cargo check" to "cargo clippy" to improve your code quality.

/// Panic Handler For Main Kernel Panic
#[cfg(target_os = "none")] // Stops false error caused by byteorder which AML depends on for testing or something
#[cfg(not(test))] // Stops false error caused by testing framework
#[cfg(not(feature = "test"))] // Stops false error caused by testing framework
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use alloc::format;

    x86_64::instructions::interrupts::disable();

    if LOGGER.try_get().is_ok() && LOGGER.get().is_some() {
        if LOGGER.get().unwrap().try_lock().is_none() {
            unsafe { LOGGER.get().unwrap().force_unlock() }
        }
        LOGGER
            .get()
            .unwrap()
            .lock()
            .error(format!("{}", info).as_str().trim());
        LOGGER.get().unwrap().lock().show_trace();
    }
    hlt_loop();
}

/// Panic Handler For Test Framework
#[cfg(feature = "test")]
#[panic_handler]
pub fn test_panic_handler(info: &core::panic::PanicInfo) -> ! {
    x86_64::instructions::interrupts::disable();
    serial_print!("[failed]");
    serial_println!("\nError: {}", info);
    if LOGGER.try_get().is_ok() && LOGGER.get().is_some() {
        if LOGGER.get().unwrap().try_lock().is_none() {
            unsafe { LOGGER.get().unwrap().force_unlock() }
        }
        LOGGER.get().unwrap().lock().show_trace();
    }
    exit_qemu(QemuExitCode::Failed);
}

/// Exit codes for QEMU emulator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

/// Exits QEMU emulator with the specified exit code
pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32)
    }
    hlt_loop();
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        serial_print!("{}... ", core::any::type_name::<T>());
        self();
        serial_println!("[Ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    for test in tests {
        test.run();
    }
}
