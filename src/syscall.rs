#[allow(unused_imports)]
use crate::{ println, serial_println};
use core::{arch::{asm}};
use x86_64::structures::idt::{ InterruptStackFrame};


type SyscallHandler = fn() -> i64;
static SYSCALL_TABLE: [Option<SyscallHandler>; 1] = [Some(write_syscall_handler)];


// Causes Strange Behavior
pub extern "x86-interrupt" fn syscall_handler(_stack_frame: InterruptStackFrame) {
    println!("SYSCALL HANDLER: Started");
    let mut syscall_numbeer: u64;
    unsafe { 
        asm!("mov {0:r}, rax", out(reg) syscall_numbeer); 
    println!("SYSCALL: NUMBER: {}", syscall_numbeer);

    if let Some(syscall_handler_fn) = SYSCALL_TABLE[syscall_numbeer as usize] {
        println!("SYSCALL HANDLER: Calling Function");
        let result = syscall_handler_fn();
         asm!("mov rax, {}", in(reg) result) 
    } else {
        println!("SYSCALL NUMBER: {} INVALID", syscall_numbeer)
    }
};
}
#[cfg(not(test))]
fn write_syscall_handler() -> i64 {
    println!("WRITE SYSCALL CALLED");
    let text_pointer: *const u8;
    let text_length: u64;
    unsafe {
        println!("SYSCALL: WRITE: Getting Text Pointer");
        asm!(
            "mov rsi, {}", out(reg) text_pointer
        );
        println!("SYSCALL: WRITE: Getting Text Length");
        asm!(
            "mov rdx, {}", out(reg) text_length
        );
    };
    println!("SYSCALL: WRITE: Dereferencing Raw Pointer");
    unsafe {
        let text: u8 = *text_pointer;
        println!("CALL println: size: {} text:{}", text_length, text);
    };

    0 // Return value of type i64
}

#[cfg(test)]
fn write_syscall_handler() -> i64 {
    serial_println!("WRITE SYSCALL CALLED");
    let text_pointer: *const u8;
    let text_length: u64;
    unsafe {
        serial_println!("SYSCALL: WRITE: Getting Text Pointer");
        asm!(
            "mov rsi, {}", out(reg) text_pointer
        );
        serial_println!("SYSCALL: WRITE: Getting Text Length");
        asm!(
            "mov rdx, {}", out(reg) text_length
        );
    };
    serial_println!("SYSCALL: WRITE: Dereferencing Raw Pointer");
    unsafe {
        let text: u8 = *text_pointer;
        serial_println!("CALL println: size: {} text:{}", text_length, text);
    };

    0 // Return value of type i64
}

pub fn test_syscall_handler_serial() {
    unsafe {
        let text = "Hello, World!\n";
        let pointer = text.as_ptr();
        let ret: i32;
        serial_println!("inside of test_syscall_handler");
        
        
        asm!("lock mov rax, {0:r}", in(reg) 0);
        asm!("lock mov rdi, {0:r}", in(reg) 1);
        asm!("lock mov rsi, {}", in(reg) pointer);
        asm!("lock mov rdx, {}", in(reg) text.len());
        serial_println!("SYSCALL");
        asm!("INT $0x80"); 
        serial_println!("SYSCALL");
        asm!("lock mov {}, rcx", out(reg) _);
        asm!("lock mov {}, r11", out(reg) _);
        asm!("lock mov {0:r}, rax", lateout(reg) ret);
        serial_println!("{}", ret);
    }
    
}


pub fn test_syscall_handler() {
    unsafe {
        let text = "Hello, World!\n";
        let pointer = text.as_ptr();
        let ret: i32;
        static mut SYSCALL_NUMBER: i32 = 1;
        
        println!("inside of test_syscall_handler");
        asm!("mov rax, {0:r}", in(reg) SYSCALL_NUMBER);
        asm!("mov rdi, {0:r}", in(reg) 1);
        asm!("mov rsi, {}", in(reg) pointer);
        asm!("mov rdx, {}", in(reg) text.len());
        println!("SYSCALL: Sending INT 0x80");

        asm!("INT $0x80"); 

        println!("SYSCALL: After Sending");
        asm!("mov {}, rcx", out(reg) _);
        asm!("mov {}, r11", out(reg) _);
        asm!("mov {0:r}, rax", lateout(reg) ret);
        println!("{}", ret);
    }
    
}