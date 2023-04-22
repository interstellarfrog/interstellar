#![no_std] // We Cannot Use The Standard Lib As It has OS Specific Functions.
#![no_main] // A Function Is Called Before The Main Function Which Sets Up The Environment So We Need To OverWrite This As We Do Not Have The OS We Are Coding One
#![feature(custom_test_frameworks)] // Allows Us To Run Custom Tests
#![test_runner(interstellar_os::test_runner)] // Defines The Test Runner Function
#![reexport_test_harness_main = "test_main"] // No Main Makes This Not Run As Behind The Scenes Main Is Called For Testing - So We Change The Name

use core::panic::PanicInfo;
use interstellar_os::println;


// Dont Mangle The Name Of This Function.
// Extern "C" means that this function uses the C calling convention.
// This Function Should Be Called _start For LLVM.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Now We Can Handle Breakpoint Exceptions Here we Cause One On Purpose");

    interstellar_os::init(); // Start Interrupt Descriptor table

    x86_64::instructions::interrupts::int3(); // Breakpoint Exception

    #[cfg(test)]
    test_main();

    println!("Now The OS Does Not Crash!");
    #[allow(clippy::empty_loop)]
    loop {}
}

#[cfg(not(test))] // If Not In Test
#[panic_handler] // This function is called on panic.
fn panic(info: &PanicInfo) -> ! { // The Panic Info Contains Information About The Panic.
    println!("{info}");
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    interstellar_os::test_panic_handler(info)
}