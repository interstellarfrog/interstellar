#![no_std] // We Cannot Use The Standard Lib As It has OS Specific Functions.
#![no_main] // A Function Is Called Before The Main Function Which Sets Up The Environment So We Need To OverWrite This As We Do Not Have The OS We Are Coding One
#![feature(custom_test_frameworks)] // Allows Us To Run Custom Tests
#![test_runner(interstellar_os::test_runner)] // Defines The Test Runner Function
#![reexport_test_harness_main = "test_main"] // No Main Makes This Not Run As Behind The Scenes Main Is Called For Testing - So We Change The Name

use core::panic::PanicInfo;
use interstellar_os::{println, memory, allocator};
use bootloader::{ BootInfo, entry_point };
use x86_64::{VirtAddr};
use interstellar_os::task::{Task, executor::Executor};
use interstellar_os::task::keyboard;
use interstellar_os::assembly::hlt_loop;

extern crate alloc;

// Auto Sets Up _start For Us
entry_point!(kernel_main); // Fixes Some Error With Types

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    interstellar_os::init(); // Start Interrupt Descriptor table ect.
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe {
        memory::init(phys_mem_offset)
    };
    let mut frame_allocator = 
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) }; 
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
    println!("Hello, World!");

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses()));
    executor.run();


    #[cfg(test)]
    test_main();

    #[allow(clippy::empty_loop)]
    hlt_loop(); // Loop Until Next Interrupt - Saves CPU Percentage
}


#[cfg(not(test))] // If Not In Test
#[panic_handler] // This function is called on panic.
fn panic(info: &PanicInfo) -> ! { // The Panic Info Contains Information About The Panic.
    println!("NOT A TEST:{info}");
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    interstellar_os::test_panic_handler(info)
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}