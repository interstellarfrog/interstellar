use alloc::vec::Vec;
use crate::vga_buffer::{WRITER, Color};
use crate::{print, println};

pub fn handle_console(line: &str) {
        let commands: Vec<&str> = line.trim().split("&&").map(str::trim).collect();
    
        for command in commands {
            let parts: Vec<&str> = command.split_whitespace().collect();
    
            if let Some((command, args)) = parts.split_first() {
                match command {
                    &"hello" => hello_command(args),
                    &"add" => add_command(args),
                    &"color" => change_color(args),
                    &"colour" => change_color(args),
                    &"echo" => echo(args),
                    &"test" => {
                        println!("\nInitializing Tests");
                        crate::tests::main();
                    },
                    &"rainbow" => {
                        WRITER.lock().rainbow_toggle();
                    },
                    &"" => {
                        return;
                    },
                    &"stack overflow" => {
                        stack_overflow();
                    },
                    _ => {
                        print!("\nERROR: UNKNOWN COMMAND - <{}>", command);
                    },
                }
            }
        }
}



//########################################
// Console Commands
//########################################



#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    volatile::Volatile::new(0).read(); // Stops The Recursion From Being Optimized
}


fn hello_command(args: &[&str]) {
    println!("\nHello, World!");
    if !args.is_empty() {
        println!("\nArguments: {:?}", args);
    }
}

fn add_command(args: &[&str]) {
    if args.len() != 2 {
        println!("\nInvalid number of arguments. Usage: add <num1> <num2>");
    } else {
        if let (Ok(num1), Ok(num2)) = (args[0].parse::<i128>(), args[1].parse::<i128>()) {
            let result = num1 + num2;
            println!("\nSum: {}", result);
        } else {
            println!("\nInvalid arguments. Usage: add <num1> <num2>");
        }
    }
}

fn change_color(args: &[&str]) {
    if args.is_empty() || args.len() != 1 {
        println!("\nInvalid Arguments. Example Usage: color <red>");
        return;
    } else {
        let arg = args[0];

        if arg == "/?" || arg == "help" {
            println!("\nExample Usage: color <red>\n\nAvailable Colors:\n
            black,
            blue,
            green,
            cyan,
            red,
            magenta,
            brown,
            lightgray,
            darkgray,
            lightblue,
            lightgreen,
            lightcyan,
            lightred,
            pink,
            yellow,
            white");
            return;
        } 
        if arg == "black" {
            WRITER.lock().change_color(Color::Black);
        } else if arg == "blue" {
            WRITER.lock().change_color(Color::Blue);
        } else if arg == "green" {
            WRITER.lock().change_color(Color::Green);
        } else if arg == "cyan" {
            WRITER.lock().change_color(Color::Cyan);
        } else if arg == "red" {
            WRITER.lock().change_color(Color::Red);
        } else if arg == "magenta" {
            WRITER.lock().change_color(Color::Magenta);
        } else if arg == "brown" {
            WRITER.lock().change_color(Color::Brown);
        } else if arg == "lightgray" {
            WRITER.lock().change_color(Color::LightGray);
        } else if arg == "darkgray" {
            WRITER.lock().change_color(Color::DarkGray);
        } else if arg == "lightblue" {
            WRITER.lock().change_color(Color::LightBlue);
        } else if arg == "lightgreen" {
            WRITER.lock().change_color(Color::LightGreen);
        } else if arg == "lightcyan" {
            WRITER.lock().change_color(Color::LightCyan);
        } else if arg == "lightred" {
            WRITER.lock().change_color(Color::LightRed);
        } else if arg == "pink" {
            WRITER.lock().change_color(Color::Pink);
        } else if arg == "yellow" {
            WRITER.lock().change_color(Color::Yellow);
        } else if arg == "white" {
            WRITER.lock().change_color(Color::White);
        } else {
            println!("\nInvalid Color: {}", arg);
        }
    }

}

fn echo(s: &[&str]) {
    println!("");
    for word in s {
        print!("{} ",word)
    }
    println!("");
}