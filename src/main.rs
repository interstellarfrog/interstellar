#![no_std] // We Cannot Use The Standard Lib As It has OS Specific Functions.
#![no_main] // A Function Is Called Before The Main Function Which Sets Up The Environment So We Need To OverWrite This As We Do Not Have The OS We Are Coding One
#![feature(custom_test_frameworks)] // Allows Us To Run Custom Tests
#![test_runner(interstellar_os::test_runner)] // Defines The Test Runner Function
#![reexport_test_harness_main = "test_main"] // No Main Makes This Not Run As Behind The Scenes Main Is Called For Testing - So We Change The Name

use core::panic::PanicInfo;
use interstellar_os::{println, memory, allocator};
use bootloader::{ BootInfo, entry_point };
use x86_64::{VirtAddr};


extern crate alloc;

use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

// Auto Sets Up _start For Us
entry_point!(kernel_main); // Fixes Some Error With Types

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Hello, World!");

    interstellar_os::init(); // Start Interrupt Descriptor table ect.

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let mut mapper = unsafe {
        memory::init(phys_mem_offset)
    };

    let mut frame_allocator = 
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) }; 

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));



    #[cfg(test)]
    test_main();

    #[allow(clippy::empty_loop)]
    interstellar_os::hlt_loop(); // Loop Until Next Interrupt - Saves CPU Percentage
}
// 0x10000201f80
#[cfg(not(test))] // If Not In Test
#[panic_handler] // This function is called on panic.
fn panic(info: &PanicInfo) -> ! { // The Panic Info Contains Information About The Panic.
    println!("NOT A TEST:{info}");
    interstellar_os::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    interstellar_os::test_panic_handler(info)
}