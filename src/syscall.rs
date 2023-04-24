#[allow(unused_imports)]
use crate::{ println, serial_println};
use core::arch::{asm};
use x86_64::structures::idt::{ InterruptStackFrame};

type SyscallHandler = fn() -> i64;
static SYSCALL_TABLE: [Option<SyscallHandler>; 1] = [Some(write_syscall_handler)];

pub extern "x86-interrupt" fn syscall_handler(_stack_frame: InterruptStackFrame) {
    let syscall_number: u64;
    unsafe { asm!("mov {}, rax", out(reg) syscall_number) }; // Might Work

    if let Some(syscall_handler_fn) = SYSCALL_TABLE[syscall_number as usize] {
        let result = syscall_handler_fn();
        unsafe { asm!("mov rax, {}", in(reg) result) }; // Might Work
    } else {
        serial_println!("SYSCALL NUMBER: {} INVALID", syscall_number)
    }
}
#[cfg(not(test))]
fn write_syscall_handler() -> i64 {
    println!("WRITE SYSCALL CALLED");
    let text_pointer: *const u8;
    let text_length: u64;
    unsafe {
        asm!(
            "mov rsi, {}", out(reg) text_pointer
        );
        asm!(
            "mov rdx, {}", out(reg) text_length
        );
    };
    unsafe {
        let text: u8 = *text_pointer;
        println!("SYSCALL: println: size: {} text:{}", text_length, text);
    };

    0 // Return value of type i64
}

#[cfg(test)]
fn write_syscall_handler() -> i64 {
    serial_println!("WRITE SYSCALL CALLED");
    let text_pointer: *const u8;
    let text_length: u64;
    unsafe {
        asm!(
            "mov {}, rsi", out(reg) text_pointer
        );
        asm!(
            "mov {}, rdx", out(reg) text_length
        );
    };
    unsafe {
        let text: u8 = *text_pointer;
        serial_println!("SYSCALL: println: size: {} text:{}", text_length, text);
    };

    0 // Return value of type i64
}

pub fn test_syscall_handler() {
    unsafe {
        let text = "Hello, World!\n";
        let pointer = text.as_ptr();
        let ret: i32;
        println!("inside of test_syscall_handler");
        
        
        asm!("lock mov rax, {0:r}", in(reg) 0);
        asm!("lock mov rdi, {0:r}", in(reg) 1);
        asm!("lock mov rsi, {}", in(reg) pointer);
        asm!("lock mov rdx, {}", in(reg) text.len());
        println!("SYSCALL");
        asm!("INT $0x80"); // CAUSES DOUBLE FAULT NO IDEA WHY
        println!("SYSCALL");
        asm!("lock mov {}, rcx", out(reg) _);
        asm!("lock mov {}, r11", out(reg) _);
        asm!("lock mov {0:r}, rax", lateout(reg) ret);
        println!("{}", ret);
    }
    
}

