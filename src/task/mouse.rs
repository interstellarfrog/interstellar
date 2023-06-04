use crate::{println};
use core::arch::asm;

pub unsafe fn mouse_init() {
    let mouse_buttons: i16;
    println!("mouse init");
    asm!("mov ax, 0");
    println!("0 in ax");
    asm!("int 0x33");
    println!("called 0x33");
    asm!("mov {0:x}, bx", out(reg) mouse_buttons);
    println!("Mouse Buttons: {}", mouse_buttons);

}



/*
const PS2_DATA_PORT: u16 = 0x60;
const PS2_COMMAND_PORT: u16 = 0x64;

fn init_mouse() {
    // Disable the keyboard by sending the appropriate command
    write_command(0xAD);

    // Enable the PS/2 controller
    write_command(0xAE);

    // Reset the mouse
    write_data(0xFF);
    wait_for_ack();
    read_data(); // discard any extra bytes sent by the mouse

    // Configure the mouse
    write_data(0xF3); // set sample rate
    wait_for_ack();
    write_data(200); // sample rate (in Hz)
    wait_for_ack();

    write_data(0xE8); // set resolution
    wait_for_ack();
    write_data(2); // resolution (2 counts/mm)
    wait_for_ack();

    write_data(0xF4); // enable reporting

    // Enable mouse interrupts
    write_command(0x20);
    let mut status = read_data();
    status |= 0b10; // enable mouse interrupts
    write_command(0x60);
    write_data(status);

    // Re-enable the keyboard
    write_command(0xAE);
}
 */