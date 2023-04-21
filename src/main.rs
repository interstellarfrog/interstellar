// We Cannot Use The Standard Lib As It has OS Specific Functions.
#![no_std]
// A Function Is Called Before The Main Function Which Sets Up The Environment.
// So We Need To OverWrite This As We Do Not Have The OS We Are Coding One.
// And We Do Not Have Access To the Rust runtime and crt0.
#![no_main]
#![feature(custom_test_frameworks)] // Allows Us To Run Custom Tests
#![test_runner(crate::test_runner)] // Defines The Test Runner Function
// No Main Makes This Not Run As Behind The Scenes Main Is Called For Testing
// So We Change The Name
#![reexport_test_harness_main = "test_main"] 

//###################
// CROSS COMPILING
//###################
// My Build
// host: x86_64-pc-windows-msvc
// release: 1.71.0-nightly

// Target Build
// To Avoid Linker Errors We want to Compile Our Code Using Our Custom Target.


mod vga_buffer;
mod serial;


use core::panic::PanicInfo;
use x86_64::instructions::port::Port;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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




#[cfg(not(test))] // If Not In Test
#[panic_handler] // This function is called on panic.
fn panic(info: &PanicInfo) -> ! { // The Panic Info Contains Information About The Panic.
    println!("{info}");
    loop {}
}

#[cfg(test)] // If Test
#[panic_handler] // This function is called on panic.
fn panic(info: &PanicInfo) -> ! { // The Panic Info Contains Information About The Panic.
    serial_println!("[Failed]\n"); // Print To Main PC Standard Out
    serial_println!("Error: {}\n", info); 
    exit_qemu(QemuExitCode::Failed);
    loop {}
}



// Dont Mangle The Name Of This Function.
// Extern "C" means that this function uses the C calling convention.
// This Function Should Be Called _start For LLVM.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    #[cfg(test)]
    test_main();

    println!("Now Has Serial Print That Can Be Used With Cargo Test");
    loop {}
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) { // Runs The Tests 
    if tests.len() == 1 {
        serial_println!("Running 1 Test")
    } else {
    println!("Running {} Tests", tests.len());
    }
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    serial_print!("trivial assertion... ");
    assert_eq!(1, 1);
    serial_println!("[ok]");
}