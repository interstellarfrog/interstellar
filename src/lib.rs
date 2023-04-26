#![no_std]
#![cfg_attr(test, no_main)] // If test - No Main
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(const_mut_refs)]

#[cfg(test)]
use bootloader::{ entry_point, BootInfo };

#[cfg(test)]
entry_point!(test_kernel_main);

use core::panic::PanicInfo;
use x86_64::instructions::{port::Port};
use core::arch::asm;

pub mod vga_buffer;
pub mod serial;
pub mod interrupts;
pub mod syscall;
pub mod gdt;
pub mod memory;
pub mod allocator;
pub mod assembly;

extern crate alloc;

/// Entry point for `cargo test`
#[cfg(test)]
#[no_mangle]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop(); // Loop Until Next Interrupt - Saves CPU Percentage
}



pub fn init() { // INITIALIZE The Interrupt Descriptor Table
    gdt::init();
    interrupts::init_idt();
    unsafe{interrupts::PICS.lock().initialize()}; // Init Hardware Interrupt Controllers
    unsafe { asm!("sti", options(nomem, nostack)) } // Enable Hardware Interrupts
}

#[inline]
pub fn hlt_loop() -> ! { // Loop Until Next Interrupt - Saves CPU Percentage
    loop {
        unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)) }
    }
}

pub trait Testable {
    fn run(&self) -> ();
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

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)] // Adds Functionallity To Enum - Debug Formatting ect.
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32)
    }
}


#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}