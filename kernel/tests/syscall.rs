#![no_std]
#![no_main]
#![feature(custom_test_frameworks)] // Allows Us To Run Custom Tests
#![test_runner(kernel::test_runner)] // Defines The Test Runner Function
#![reexport_test_harness_main = "test_main"]

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
use kernel::init;
use kernel::syscall::test_syscall_handler_serial;

entry_point!(system_call_test_main);

fn system_call_test_main(boot_info: &'static mut BootInfo) -> ! {
    init(boot_info);

    test_main();
    #[allow(clippy::empty_loop)]
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info)
}

#[test_case]
fn test_write() {
    test_syscall_handler_serial();
}
