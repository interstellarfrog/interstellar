#![no_std]
#![no_main]
#![feature(custom_test_frameworks)] // Allows Us To Run Custom Tests
#![test_runner(interstellar_os::test_runner)] // Defines The Test Runner Function
#![reexport_test_harness_main = "test_main"] 

use core::panic::PanicInfo;
use interstellar_os::syscall::test_syscall_handler;

#[no_mangle] // Dont Mangle The Name Of This Function
pub extern "C" fn _start() -> ! {
    test_main();
    #[allow(clippy::empty_loop)]
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    interstellar_os::test_panic_handler(info)
}

#[test_case]
fn test_write() {
    test_syscall_handler();
}