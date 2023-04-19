// We Cannot Use The Standard Lib As It has OS Specific Functions.
#![no_std]

// A Function Is Called Before The Main Function Which Sets Up The Environment.
// So We Need To OverWrite This As We Do Not Have The OS We Are Coding One.
// And We Do Not Have Access To the Rust runtime and crt0.
#![no_main]

//###################
// CROSS COMPILING
//###################
// My Build
//host: x86_64-pc-windows-msvc
//release: 1.71.0-nightly

// Target Build
// To Avoid Linker Errors We want to Compile Our Code Using --target thumbv7em-none-eabihf.
// This Target Is For An Embedded ARM System Which Does Not Really Matter As All We Need Is A Target That Does Not Have An OS.
// rustup target add thumbv7em-none-eabihf
// cargo build --target thumbv7em-none-eabihf

use core::panic::PanicInfo;

// This function is called on panic.
// The Panic Info Contains Information About The Panic.
// The ! means that this function will never return.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
// Dont Mangle The Name Of This Function.
// Extern C means that this function uses the C calling convention.
// This Function Should Be Called _start For LLVM.
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // Raw Pointer To The VGA Buffer.
    let vga_buffer = 0xb8000 as *mut u8;
    // For Every Character In The String We Want To Print.
    for (i, &byte) in "Hello World!".as_bytes().iter().enumerate() {
        unsafe {
            // Write The Character To The VGA Buffer.
            *vga_buffer.offset(i as isize * 2) = byte;
            // Write The Color Code To The VGA Buffer 0xb = Light Cyan
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;

        }
    }
    loop {}
}



