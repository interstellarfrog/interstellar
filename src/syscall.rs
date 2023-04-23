use crate::{ println, serial_println};
use core::arch::{asm};
use x86_64::structures::idt::{ InterruptStackFrame};

type SyscallHandler = fn() -> i64;
static SYSCALL_TABLE: [Option<SyscallHandler>; 1] = [Some(write_syscall_handler)];

pub extern "x86-interrupt" fn syscall_handler(stack_frame: InterruptStackFrame) {
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
        println!("{}", text);
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
        serial_println!("{}", text);
    };

    0 // Return value of type i64
}

#[test_case]
pub fn test_syscall_handler() {
    unsafe {
        let text = "Hello, World!\n";
        let pointer = text.as_ptr();
        let ret: i32;
        serial_println!("inside of test_syscall_handler");
        
        
        asm!("mov rax, {0:r}", in(reg) 0);
        asm!("mov rdi, {0:r}", in(reg) 1);
        asm!("mov rsi, {}", in(reg) pointer);
        asm!("mov rdx, {}", in(reg) text.len());
        serial_println!("SYSCALL");
        asm!("int 0x80"); // BREAKS THE CODE PROBABLY BECAUSE INTERRUPTS NOT TURNED ON YET
        serial_println!("SYSCALL");
        asm!("mov {}, rcx", out(reg) _);
        asm!("mov {}, r11", out(reg) _);
        asm!("mov {0:r}, rax", lateout(reg) ret);
        serial_println!("{}", ret);
    }
    
}
