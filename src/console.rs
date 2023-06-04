use alloc::string::String;
use crate::vga_buffer::WRITER;
use crate::{print, println};

pub fn handle_console(command: &String) {
    let c = command.trim();

    match c {
        "test" => {
            println!("\nInitializing Tests");
            crate::tests::main();
    },
        "rainbow" => {
            WRITER.lock().rainbow_toggle();
        },
        "" => {
            return;
        },
        "stack overflow" => {
            stack_overflow();
        },
        _ => {
            print!("\nERROR: UNKNOWN COMMAND - <{}>", c);
        },
    };



}


#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read(); // Stops The Recursion From Being Optimized
}