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

use pic8259::ChainedPics;
use x86_64::{
    instructions::port::{Port, PortReadOnly},
    registers::control::Cr2,
    structures::idt::{InterruptStackFrame, PageFaultErrorCode},
};

use crate::other::log::LOGGER;

//###############################################
//        Exception handlers
//###############################################

/// Handler for the divide by zero exception
pub extern "x86-interrupt" fn divide_by_zero_fault_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DIVIDE BY ZERO\n{:#?}", stack_frame);
}

/// Handler for the debug exception
pub extern "x86-interrupt" fn debug_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DEBUG\n{:#?}", stack_frame);
}

/// Handler for the non-maskable interrupt exception
pub extern "x86-interrupt" fn non_masked_interrupt_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: NON-MASKABLE INTERRUPT\n{:#?}", stack_frame);
}

/// Handler for the breakpoint exception
pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    unsafe { LOGGER.get().unwrap().force_unlock() };
    LOGGER
        .get()
        .unwrap()
        .lock()
        .error(&alloc::format!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame));
}

/// Handler for the overflow exception
pub extern "x86-interrupt" fn overflow_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: OVERFLOW\n{:#?}", stack_frame);
}

/// Handler for the bound-range-exceeded exception
pub extern "x86-interrupt" fn bound_range_exceeded_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: BOUND_RANGE_EXCEEDED\n{:#?}", stack_frame);
}

/// Handler for the invalid opcode exception
pub extern "x86-interrupt" fn invalid_opcode_fault_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: INVALID OPCODE\n{:#?}", stack_frame);
}

/// Handler for the device-not-available exception
pub extern "x86-interrupt" fn device_not_available_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: DEVICE-NOT-AVAILABLE\n{:#?}", stack_frame);
}

/// Handler for the double fault exception
pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

/// Handler for the invalid TSS (Task State Segment) exception
pub extern "x86-interrupt" fn invalid_tss_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: INVALID TSS\nError Code: {:?}\n{:#?}",
        error_code, stack_frame
    );
}

/// Handler for the segment-not-present exception
pub extern "x86-interrupt" fn segment_not_present_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: SEGMENT-NOT-PRESENT\nError Code: {:?}\n{:#?}",
        error_code, stack_frame
    );
}

/// Handler for the stack-segment-fault exception
pub extern "x86-interrupt" fn stack_segment_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("EXCEPTION: STACK-SEGMENT-FAULT\n{:#?}", stack_frame);
}

/// Handler for the general protection fault exception
pub extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    stack_segment: u64,
) {
    panic!(
        "EXCEPTION: GENERAL PROTECTION FAULT\n{:#?}\nStack Segment: {}",
        stack_frame, stack_segment
    );
}

/// Handler for the page-fault exception
pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let address = Cr2::read();
    let protv = error_code.contains(PageFaultErrorCode::PROTECTION_VIOLATION);
    let write = error_code.contains(PageFaultErrorCode::CAUSED_BY_WRITE);
    let user = error_code.contains(PageFaultErrorCode::USER_MODE);
    let malformed = error_code.contains(PageFaultErrorCode::MALFORMED_TABLE);
    let ins = error_code.contains(PageFaultErrorCode::INSTRUCTION_FETCH);

    panic!(
        "EXCEPTION: PAGE FAULT ({}{}{}{}{}at 0x{:x?})\n{:#?}",
        if protv { "protection-violation " } else { "" },
        if write { "read-only " } else { "" },
        if user { "user-mode " } else { "" },
        if malformed { "reserved " } else { "" },
        if ins { "fetch " } else { "" },
        address.as_u64(),
        stack_frame
    );
}

/// Handler for the x87-floating-point exception
pub extern "x86-interrupt" fn x87_floating_point_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: x87-FLOATING-POINT\n{:#?}", stack_frame);
}

/// Handler for the alignment-check exception
pub extern "x86-interrupt" fn alignment_check_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: ALIGNMENT-CHECK\nError Code: {:?}\n{:#?}",
        error_code, stack_frame
    );
}

/// Handler for the machine-check exception
pub extern "x86-interrupt" fn machine_check_handler(stack_frame: InterruptStackFrame) -> ! {
    panic!("EXCEPTION: MACHINE-CHECK\n{:#?}", stack_frame);
}

/// Handler for the simd-floating-point exception
pub extern "x86-interrupt" fn simd_floating_point_handler(stack_frame: InterruptStackFrame) {
    panic!("EXCEPTION: SIMD-FLOATING-POINT\n{:#?}", stack_frame);
}

/// Handler for the vmm-communication exception
pub extern "x86-interrupt" fn vmm_communication_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!(
        "EXCEPTION: VMM-COMMUNICATION\nError Code: {:?}\n{:#?}",
        error_code, stack_frame
    );
}

/// Handler for the security exception
pub extern "x86-interrupt" fn security_exception_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) {
    panic!("EXCEPTION: SECURITY EXCEPTION\n{:#?}", stack_frame);
}

//###############################################
//        Interrupt Indexes
//###############################################

pub const IOAPIC_INTERRUPT_INDEX_OFFSET: u8 = 32;

pub const LAPIC_INTERRUPT_INDEX_OFFSET: u8 = 46;

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    _IoApic = IOAPIC_INTERRUPT_INDEX_OFFSET,  // 32
    Pit,                                      // 0/33
    Keyboard,                                 // 1/34
    _CascadeForSecondPic,                     // 2/35
    _SerialPort2Controller,                   // 3/36
    _SerialPort1Controller,                   // 4/37
    _ParallelPort2And3OrSoundCard,            // 5/38
    _FloppyDiskController,                    // 6/39
    _ParallelPort1OrPrinter,                  // 7/40
    _RTC,                                     // 8/41
    _GeneralIOAndSound,                       // 9/42
    _ACPI,                                    // 10/43
    _USBAndNetwork,                           // 11/44
    Mouse,                                    // 12/45
    ApicError = LAPIC_INTERRUPT_INDEX_OFFSET, // 46
    Timer,                                    // 47
    Spurious,                                 // 48
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum IoApicTableIndex {
    // These can be different depending on the UEFI software but most map them 1:1 the same as the PICS
    Pit = 0, // For some reason this is 2 instead of 0 for QEMUs IOAPIC but we use the interrupt source overide table entries to remap it
    Keyboard = 1,
    Mouse = 12,
}

impl From<IoApicTableIndex> for u8 {
    fn from(val: IoApicTableIndex) -> Self {
        val as u8
    }
}

impl From<IoApicTableIndex> for usize {
    fn from(val: IoApicTableIndex) -> Self {
        val as usize
    }
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

/// Handler for the PIT interrupt
pub extern "x86-interrupt" fn pit_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe { crate::time::PIT_COUNT.fetch_add(1, core::sync::atomic::Ordering::SeqCst) };

    unsafe {
        if let Some(mut apic) = super::LAPIC.get().unwrap().try_lock() {
            apic.end_of_interrupt();
        } else {
            super::LAPIC.get().unwrap().force_unlock();
            super::LAPIC.get().unwrap().lock().end_of_interrupt();
        }
    } // Tell It We Are Done
}

//###############################################
//        Local APIC interrupt handlers
//###############################################

pub extern "x86-interrupt" fn error_interrupt_handler(stack_frame: InterruptStackFrame) {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .error(&alloc::format!("APIC ERROR: {:#?}", stack_frame));
    unsafe {
        if let Some(mut apic) = super::LAPIC.get().unwrap().try_lock() {
            apic.end_of_interrupt();
        } else {
            super::LAPIC.get().unwrap().force_unlock();
            super::LAPIC.get().unwrap().lock().end_of_interrupt();
        }
    }
}

pub extern "x86-interrupt" fn apic_timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe { crate::time::APIC_COUNT.fetch_add(1, core::sync::atomic::Ordering::SeqCst) };
    unsafe {
        if let Some(mut apic) = super::LAPIC.get().unwrap().try_lock() {
            apic.end_of_interrupt();
        } else {
            super::LAPIC.get().unwrap().force_unlock();
            super::LAPIC.get().unwrap().lock().end_of_interrupt();
        }
    }
}

pub extern "x86-interrupt" fn spurious_interrupt_handler(stack_frame: InterruptStackFrame) {
    LOGGER.get().unwrap().lock().error(&alloc::format!(
        "SPURIOUS HARDWARE ERROR: {:#?}",
        stack_frame
    ));
    unsafe {
        if let Some(mut apic) = super::LAPIC.get().unwrap().try_lock() {
            apic.end_of_interrupt();
        } else {
            super::LAPIC.get().unwrap().force_unlock();
            super::LAPIC.get().unwrap().lock().end_of_interrupt();
        }
    }
}

//###############################################
//        IOAPIC interrupt handlers
//###############################################

/// Handler for the keyboard interrupt
pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60); // Get PS2 Data Port
    let scancode: u8 = unsafe { port.read() }; // Read Scan Code From Port
    crate::task::keyboard::add_scancode(scancode);
    unsafe {
        super::LAPIC.get().unwrap().lock().end_of_interrupt();
    } // Tell It We Are Done
}

/// Handler for the mouse interrupt
pub extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = PortReadOnly::new(0x60);
    let packet = unsafe { port.read() };
    crate::task::mouse::write(packet);
    unsafe {
        super::LAPIC.get().unwrap().lock().end_of_interrupt();
    } // Tell It We Are Done
}
