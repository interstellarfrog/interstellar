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

use crate::other::log::{Logger, BACKTRACE, BACKTRACE_INDEX, MAX_BACKTRACE_ENTRIES};
use bootloader_api::BootInfo as BI;
use core::env;
use drivers::fs::initrd;
use drivers::screen::framebuffer::{FrameBufferWriter, FRAMEBUFFER};
use other::assembly::hlt_loop;
use other::info::{BootInfo, Info, BOOT_INFO, INFO};
use other::log::{LogLevel, LOGGER};
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
pub mod interrupts;
//pub mod memory;
pub mod memory;
//pub mod allocator;
pub mod allocator;

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
/// Interrupt Descriptor Table (IDT), Programmable Interrupt Controllers (PICs), enabling interrupts, and performing other necessary initialization
pub fn init(boot_info: &'static mut BI) {
    #[cfg(not(debug_assertions))]
    #[cfg(not(feature = "test"))]
    // Initialize the logger
    LOGGER.init_once(|| {
        let screen_printing = true;
        let serial_printing = false;
        let log_level = LogLevel::Normal;
        let tracing = true;

        spinning_top::Spinlock::new(Logger {
            screen_printing,
            serial_printing,
            log_level,
            tracing,
        })
    });

    #[cfg(debug_assertions)]
    #[cfg(not(feature = "test"))]
    // Initialize the logger
    LOGGER.init_once(|| {
        let screen_printing = true;
        let serial_printing = true;
        let log_level = LogLevel::High;
        let tracing = true;

        spinning_top::Spinlock::new(Logger {
            screen_printing,
            serial_printing,
            log_level,
            tracing,
        })
    });

    #[cfg(feature = "test")]
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

    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Initializing info"), file!(), line!());

    BOOT_INFO.init_once(|| {
        let api_version = boot_info.api_version;
        let mut framebuffer_info = None;
        if let Some(fb) = boot_info.framebuffer.as_mut() {
            framebuffer_info = Some(fb.info());
        }
        let physical_memory_offset = boot_info.physical_memory_offset.as_mut().cloned();
        let recursive_index = boot_info.recursive_index.as_mut().cloned();
        let rsdp_addr = boot_info.rsdp_addr.as_mut().cloned();
        let tls_template = boot_info.tls_template.as_mut().cloned();
        let ramdisk_addr = boot_info.ramdisk_addr.as_mut().cloned();
        let ramdisk_len = boot_info.ramdisk_len;

        spinning_top::Spinlock::new(BootInfo {
            api_version,
            framebuffer_info,
            physical_memory_offset,
            recursive_index,
            rsdp_addr,
            tls_template,
            ramdisk_addr,
            ramdisk_len,
        })
    });

    INFO.init_once(|| {
        let os_version = env!("CARGO_PKG_VERSION", "Could not get OS version");

        spinning_top::Spinlock::new(Info { os_version })
    });

    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Initializing framebuffer"), file!(), line!());

    FRAMEBUFFER.init_once(|| {
        if let Some(info) = BOOT_INFO.get().unwrap().lock().framebuffer_info {
            if let Some(buffer) = boot_info.framebuffer.as_mut() {
                spinning_top::Spinlock::new(FrameBufferWriter::new(buffer.buffer_mut(), info))
            } else {
                panic!("BOOTLOADER NOT CONFIGURED TO SUPPORT FRAMEBUFFER");
            }
        } else {
            panic!("BOOTLOADER NOT CONFIGURED TO SUPPORT FRAMEBUFFER");
        }
    });

    LOGGER.get().unwrap().lock().info("Initializing Memory");

    if let bootloader_api::info::Optional::Some(phys_mem_offset) = boot_info.physical_memory_offset
    {
        allocator::init(Some(phys_mem_offset), &mut boot_info.memory_regions);
    } else {
        panic!("No physical memory offset given from bootloader");
    }

    LOGGER.get().unwrap().lock().info(&alloc::format!(
        "total memory: {}",
        memory::MEMORY.lock().as_ref().unwrap().total_memory
    ));
    LOGGER.get().unwrap().lock().info(&alloc::format!(
        "total memory: {}GB",
        memory::MEMORY
            .lock()
            .as_ref()
            .unwrap()
            .total_mem_gigabytes()
    ));

    let ramdisk_addr = BOOT_INFO.get().unwrap().lock().ramdisk_addr; // Stops deadlock

    if let Some(ramdisk_addr) = ramdisk_addr {
        let ramdisk_len = BOOT_INFO.get().unwrap().lock().ramdisk_len;
        unsafe { initrd::init(ramdisk_addr as *const u8, ramdisk_len) };
    } else {
        LOGGER.get().unwrap().lock().info("No initrd found");
    }

    gdt::init();

    let acpi_tables = acpi::init(PhysAddr::new(boot_info.rsdp_addr.into_option().unwrap()));

    let acpi_platform_info = acpi_tables.platform_info().unwrap();

    interrupts::init(&acpi_platform_info);

    drivers::hid::keyboard::init();

    drivers::hid::mouse::init();

    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Initialized kernel"), file!(), line!());
}

/// Allocation error handler
#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

// All below false errors are mostly for VScode as rust analyzer runs commands like cargo check --workspace or cargo clippy --workspace.
// I recommend changing your cargo check to cargo clippy to improve your code quality.

#[cfg(target_os = "none")] // Stops false error caused by byteorder which AML depends on for testing or something
#[cfg(not(test))] // Stops false error caused by testing framework
#[cfg(not(feature = "test"))]
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
