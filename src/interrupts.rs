// The Code In This File Is For Handling CPU Exceptions And Interrupts - 0 Division errors ect. And Keyboard Input ect.
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::instructions::port::Port;
use x86_64::registers::control::Cr2;
use crate::{println, print, gdt, hlt_loop};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::{self, Mutex};
use pc_keyboard::{ layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1 };

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard, // PIC_1_OFFSET + 1
}

lazy_static! { // Needs To Live For Life Of Program And Only Init When Needed
    
    static ref IDT: InterruptDescriptorTable = {
        
        let mut idt = InterruptDescriptorTable::new(); // Make New IDT
        idt.breakpoint.set_handler_fn(breakpoint_handler); // Breakpoint Exception Handler
        idt.double_fault.set_handler_fn(double_fault_handler); // Double Fault Handler
        //idt.divide_error.set_handler_fn(divide_by_zero_handler);
        //idt.invalid_opcode.set_handler_fn(invalif_opcode_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        unsafe { idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX); } // Set Stack To Switch To
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt.non_maskable_interrupt.set_handler_fn(non_masked_interrupt_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt); // Timer Interrupt Handler
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler); // Keyboard Interrupt Handler
        idt[80].set_handler_fn(crate::syscall::syscall_handler); // When 0x80 Called 
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame, _error_code: u64) -> ! // Cannot Return From A Double Fault
{
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern  "x86-interrupt" fn general_protection_fault_handler(stack_frame: InterruptStackFrame, _something: u64,) {
    println!("EXCEPTION: GENERAL PROTECTION FAULT");
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern  "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode,) {
    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read()); // CR2 Contains Accessed Virtual Address 
    println!("Error Code: {:?}", error_code); // Extra Info
    println!("{:#?}", stack_frame);
    hlt_loop();
}



extern  "x86-interrupt" fn non_masked_interrupt_handler(stack_frame: InterruptStackFrame,) {
    println!("EXCEPTION: Non Masked Interrupt");
    println!("{:#?}", stack_frame);
    hlt_loop();
}



pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });



impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

extern "x86-interrupt" fn timer_interrupt(_stack_frame: InterruptStackFrame) {
    print!(".");
    unsafe {PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8())} // Tell It We Are Done
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {


    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Uk105Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(ScancodeSet1::new(), layouts::Uk105Key  ,HandleControl::Ignore));
    }

    let mut keyboard = KEYBOARD.lock(); // Get Lock
    let mut port = Port::new(0x60); // Get PS2 Data Port 
    let scancode: u8 = unsafe { port.read() }; // Read Scan Code From Port


    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) { // Gets The Key Event
        if let Some(key) = keyboard.process_keyevent(key_event) { // Turns Key Events Into Unicode And Handles If A Character Is Capital Or Not
            match key {
                DecodedKey::Unicode(character) => print!("{}", character), // If Unicode
                DecodedKey::RawKey(key) => print!("{:?}", key), // If Raw Debug Print
            }
        }
    }
    unsafe {PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8())} // Tell It We Are Done
}



#[test_case]
fn test_breakpoint_exception() {
    x86_64::instructions::interrupts::int3(); // Cause Breakpoint Exception
}