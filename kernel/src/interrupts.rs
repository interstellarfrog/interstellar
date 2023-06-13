//Copyright (C) <2023>  <interstellarfrog>
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


/// The Code In This File Is For Handling CPU Exceptions And Interrupts - 0 Division errors etc. And Keyboard Input etc.
use x86_64::{instructions::port::{Port, PortReadOnly}, structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode}};
use crate::{gdt, hlt_loop, serial_println};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::{self};
use x86_64::registers::control::Cr2;

/// Enum representing the indices of different interrupts
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard, // PIC_1_OFFSET + 1
    Mouse = PIC_1_OFFSET + 12,
    SYSCALL = 80_u8,
}

// Lazy-static IDT (Interrupt Descriptor Table) for handling interrupts
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new(); // Create a new IDT
        idt.breakpoint.set_handler_fn(breakpoint_handler); // Set the handler for the breakpoint exception
        idt.double_fault.set_handler_fn(double_fault_handler); // Set the handler for the double fault exception
        idt.divide_error.set_handler_fn(divide_by_zero_fault_handler); // Set the handler for the divide by zero exception
        idt.invalid_opcode.set_handler_fn(invalid_opcode_fault_handler); // Set the handler for the invalid opcode exception
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler); // Set the handler for the general protection fault exception
        idt.invalid_tss.set_handler_fn(invalid_tss_fault_handler); // Set the handler for the invalid TSS (Task State Segment) exception
        idt.security_exception.set_handler_fn(security_exception_fault_handler); // Set the handler for the security exception
        unsafe {
            // Set the stack index for the double fault handler to switch the stack
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt.page_fault.set_handler_fn(page_fault_handler); // Set the handler for the page fault exception
        idt.non_maskable_interrupt.set_handler_fn(non_masked_interrupt_handler); // Set the handler for the non-maskable interrupt exception
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt); // Set the handler for the timer interrupt
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler); // Set the handler for the keyboard interrupt
        idt[InterruptIndex::Mouse.as_usize()].set_handler_fn(mouse_interrupt_handler); // Set the handler for the mouse interrupt
        //idt[0x80].set_handler_fn(crate::syscall::syscall_handler); // Set the handler for the syscall interrupt (0x80)
        idt
    };
}

/// Initialize the IDT
pub fn init_idt() {
    IDT.load();
}


/// Handler for the breakpoint exception
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

/// Handler for the double fault exception
extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

/// Handler for the divide by zero exception
extern "x86-interrupt" fn divide_by_zero_fault_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
}

/// Handler for the invalid opcode exception
extern "x86-interrupt" fn invalid_opcode_fault_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
}

/// Handler for the general protection fault exception
extern "x86-interrupt" fn general_protection_fault_handler(stack_frame: InterruptStackFrame, stack_segment: u64) {
    serial_println!("EXCEPTION: GENERAL PROTECTION FAULT");
    serial_println!("{:#?}", stack_frame);
    serial_println!("Stack Segment: {}", stack_segment);
    hlt_loop();
}

/// Handler for the invalid TSS (Task State Segment) exception
extern "x86-interrupt" fn invalid_tss_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
    panic!("EXCEPTION: INVALID TSS\n{:#?}", stack_frame);
}

/// Handler for the security exception
extern "x86-interrupt" fn security_exception_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) {
    panic!("EXCEPTION: SECURITY EXCEPTION\n{:#?}", stack_frame);
}

/// Handler for the page fault exception
extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    serial_println!("EXCEPTION: PAGE FAULT");
    serial_println!("Accessed Address: {:?}", Cr2::read()); // CR2 Contains Accessed Virtual Address 
    serial_println!("Error Code: {:?}", error_code); // Extra Info
    serial_println!("{:#?}", stack_frame);
    hlt_loop();
}

/// Handler for the non-maskable interrupt exception
extern "x86-interrupt" fn non_masked_interrupt_handler(stack_frame: InterruptStackFrame) {
    serial_println!("EXCEPTION: NON-MASKABLE INTERRUPT");
    serial_println!("{:#?}", stack_frame);
    hlt_loop();
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}


//###############################################
//        Hardware Interrupts
//###############################################

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });



/// Handler for the timer interrupt
extern "x86-interrupt" fn timer_interrupt(_stack_frame: InterruptStackFrame) {
    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8()) } // Tell It We Are Done
}

/// Handler for the keyboard interrupt
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60); // Get PS2 Data Port
    let scancode: u8 = unsafe { port.read() }; // Read Scan Code From Port
    crate::task::keyboard::add_scancode(scancode);
    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8()) } // Tell It We Are Done
}

/// Handler for the mouse interrupt
extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = PortReadOnly::new(0x60);
    let packet = unsafe { port.read() };
    crate::task::mouse::write(packet);
    unsafe { PICS.lock().notify_end_of_interrupt(InterruptIndex::Mouse.as_u8()) }
}