//This file contains code for interstellar OS - https://github.com/interstellarfrog/interstellar
//Copyright (C) 2023  contributors of the interstellar OS project
//
//This program is free software: you can redistribute it and/or modify
//it under the terms of the GNU General Public License as published by
//the Free Software Foundation, either version 3 of the License, or
//(at your option) any later version.
//
//This program is distributed in the hope that it will be useful,
//but WITHOUT ANY WARRANTY; without even the implied warranty of
//MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//GNU General Public License for more details.
//
//You should have received a copy of the GNU General Public License
//along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::other::log::LOGGER;
use core::arch::asm;
use x86_64::structures::idt::InterruptStackFrame;

type SyscallHandler = fn() -> i64;
static SYSCALL_TABLE: [Option<SyscallHandler>; 1] = [Some(write_syscall_handler)];

pub extern "x86-interrupt" fn syscall_handler(_stack_frame: InterruptStackFrame) {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Handling system call", file!(), line!());

    let mut syscall_number: i32;
    unsafe {
        asm!("mov {0:r}, rax", out(reg) syscall_number);
        if syscall_number != 1 {
        } else if let Some(syscall_handler_fn) = SYSCALL_TABLE[syscall_number as usize] {
            let result = syscall_handler_fn();
            asm!("mov rax, {}", in(reg) result)
        }
    }
}

fn write_syscall_handler() -> i64 {
    let text_pointer: *const u8;
    let _text_length: u64;
    unsafe {
        asm!(
            "mov rsi, {}", out(reg) text_pointer
        );
        asm!(
            "mov rdx, {}", out(reg) _text_length
        );
    };
    unsafe {
        let _text: u8 = *text_pointer;
    };

    0 // Return value of type i64
}

pub fn test_syscall_handler_serial() {
    unsafe {
        let text = "Hello, World!\n";
        let pointer = text.as_ptr();
        let _ret: i32;

        asm!("lock mov rax, {0:r}", in(reg) 0);
        asm!("lock mov rdi, {0:r}", in(reg) 1);
        asm!("lock mov rsi, {}", in(reg) pointer);
        asm!("lock mov rdx, {}", in(reg) text.len());
        asm!("INT $0x80");
        asm!("lock mov {}, rcx", out(reg) _);
        asm!("lock mov {}, r11", out(reg) _);
        asm!("lock mov {0:r}, rax", lateout(reg) _ret);
    }
}

pub fn test_syscall_handler() {
    unsafe {
        let text = "Hello, World!\n";
        let pointer = text.as_ptr();
        let _ret: i32;
        static mut SYSCALL_NUMBER: i32 = 1;

        //println!("inside of test_syscall_handler");
        asm!("mov rax, {0:r}", in(reg) SYSCALL_NUMBER);
        asm!("mov rdi, {0:r}", in(reg) 1);
        asm!("mov rsi, {}", in(reg) pointer);
        asm!("mov rdx, {}", in(reg) text.len());
        //println!("SYSCALL: Sending INT 0x80");

        asm!("INT $0x80");

        //println!("SYSCALL: After Sending");
        asm!("mov {}, rcx", out(reg) _);
        asm!("mov {}, r11", out(reg) _);
        asm!("mov {0:r}, rax", lateout(reg) _ret);
        //println!("{}", ret);
    }
}

pub fn new_test_syscall_handler() {
    let buf = "Hello From Asm!\n";
    let _ret: i32;
    unsafe {
        asm!("int 0x80",
            in("rax") 1,
            in("rdi") 1,
            in("rsi") buf.as_ptr(),
            in("rdx") buf.len(),
            out("rcx") _,
            out("r11") _,
            lateout("rax") _ret,
        );
    }
}
