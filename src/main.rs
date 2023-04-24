#![no_std] // We Cannot Use The Standard Lib As It has OS Specific Functions.
#![no_main] // A Function Is Called Before The Main Function Which Sets Up The Environment So We Need To OverWrite This As We Do Not Have The OS We Are Coding One
#![feature(custom_test_frameworks)] // Allows Us To Run Custom Tests
#![test_runner(interstellar_os::test_runner)] // Defines The Test Runner Function
#![reexport_test_harness_main = "test_main"] // No Main Makes This Not Run As Behind The Scenes Main Is Called For Testing - So We Change The Name

use core::panic::PanicInfo;
use interstellar_os::println;
use bootloader::{ BootInfo, entry_point };
use x86_64::{VirtAddr, structures::paging::Page};
use interstellar_os::memory;

// Auto Sets Up _start For Us
entry_point!(kernel_main); // Fixes Some Error With Types

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Now We Can Map Virtual Memory To Physical Memory And Allocate Frames Here We Map The Address 0xdeadbeef000 To 0xb8000 To Print New! To The VGA Buffer");

    interstellar_os::init(); // Start Interrupt Descriptor table

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    let mut mapper = unsafe {
        memory::init(phys_mem_offset)
    };

    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map) 
    }; 

    //Map Unused Page
    let page = Page::containing_address(VirtAddr::new(0xdeadbeef000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator); // Map Virt Address 0 To 0xb8000

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    //We don’t write to the start of the page because the top line of the VGA buffer is directly shifted off the screen by the next println
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) }; // 

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