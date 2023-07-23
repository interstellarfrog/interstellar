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

use crate::other::log::LOGGER;
use crate::{
    drivers::screen::framebuffer::{Color, FRAMEBUFFER},
    print, println,
    memory::MEMORY,
};
use alloc::vec::Vec;

/// Handles the console input by executing the corresponding commands.
///
/// # Arguments
///
/// * `line` - The input line from the console.
pub fn handle_console(line: &str) -> bool {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Handling console command"), file!(), line!());
    let commands: Vec<&str> = line.trim().split("&&").map(str::trim).collect();

    for command in commands {
        let parts: Vec<&str> = command.split_whitespace().collect();

        if let Some((command, args)) = parts.split_first() {
            match *command {
                "hello" => hello_command(args),
                "add" => add_command(args),
                "subtract" => subtract_command(args),
                "multiply" => multiply_command(args),
                "divide" => divide_command(args),
                "power" => power_command(args),
                "mem" => check_memory(),
                "color" => change_color(args),
                "bgcolor" => change_background_color(args),
                "colour" => change_color(args),
                "echo" => echo(args),
                "help" => help_command(),
                "test" => {
                    println!("\nInitializing Tests");
                    crate::other::tests::main();
                }
                "cls" => {
                    FRAMEBUFFER.get().unwrap().lock().clear();
                    return true;
                }
                "clear" => {
                    FRAMEBUFFER.get().unwrap().lock().clear();
                    return true;
                }
                "rainbow" => {
                    // WRITER.lock().rainbow_toggle();
                }
                "" => {
                    return false;
                }
                "stack_overflow" => {
                    stack_overflow();
                }
                _ => {
                    print!("\nERROR: UNKNOWN COMMAND - <{}>", command);
                }
            }
        }
    }

    false
}

//########################################
// Console Commands
//########################################

/// Generates a stack overflow by calling itself recursively.
#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow();
    unsafe { volatile::VolatilePtr::new((&mut 0x0).into()) }; // Stops The Recursion From Being Optimized
}

/// Executes the "hello" command.
///
/// # Arguments
///
/// * `args` - The arguments passed to the command.
fn hello_command(args: &[&str]) {
    println!("\nHello, World!");
    if !args.is_empty() {
        println!("\nArguments: {:?}", args);
    }
}

/// Executes the "add" command.
///
/// # Arguments
///
/// * `args` - The arguments passed to the command.
fn add_command(args: &[&str]) {
    if args.len() != 2 {
        println!("\nInvalid number of arguments. Usage: add <num1> <num2>");
    } else if let (Ok(num1), Ok(num2)) = (args[0].parse::<i128>(), args[1].parse::<i128>()) {
        let result = num1 + num2;
        println!("\nSum: {}", result);
    } else {
        println!("\nInvalid arguments. Usage: add <num1> <num2>");
    }
}

/// Executes the "subtract" command.
///
/// # Arguments
///
/// * `args` - The arguments passed to the command.
fn subtract_command(args: &[&str]) {
    if args.len() != 2 {
        println!("\nInvalid number of arguments. Usage: subtract <num1> <num2>");
    } else if let (Ok(num1), Ok(num2)) = (args[0].parse::<i128>(), args[1].parse::<i128>()) {
        let result = num1 - num2;
        println!("\nDifference: {}", result);
    } else {
        println!("\nInvalid arguments. Usage: subtract <num1> <num2>");
    }
}

/// Executes the "multiply" command.
///
/// # Arguments
///
/// * `args` - The arguments passed to the command.
fn multiply_command(args: &[&str]) {
    if args.len() != 2 {
        println!("\nInvalid number of arguments. Usage: multiply <num1> <num2>");
    } else if let (Ok(num1), Ok(num2)) = (args[0].parse::<i128>(), args[1].parse::<i128>()) {
        let result = num1 * num2;
        println!("\nProduct: {}", result);
    } else {
        println!("\nInvalid arguments. Usage: multiply <num1> <num2>");
    }
}

/// Executes the "divide" command.
///
/// # Arguments
///
/// * `args` - The arguments passed to the command.
fn divide_command(args: &[&str]) {
    if args.len() != 2 {
        println!("\nInvalid number of arguments. Usage: divide <num1> <num2>");
    } else if let (Ok(num1), Ok(num2)) = (args[0].parse::<f64>(), args[1].parse::<f64>()) {
        if num2 == 0.0 {
            println!("\nCannot divide by zero.");
        } else {
            let result = num1 / num2;
            println!("\nQuotient: {}", result);
        }
    } else {
        println!("\nInvalid arguments. Usage: divide <num1> <num2>");
    }
}

/// Executes the "power" command.
///
/// # Arguments
///
/// * `args` - The arguments passed to the command.
fn power_command(args: &[&str]) {
    if args.len() != 2 {
        println!("\nInvalid number of arguments. Usage: power <base> <exponent>");
    } else if let (Ok(base), Ok(exponent)) = (args[0].parse::<f64>(), args[1].parse::<f64>()) {
        let result = power(base, exponent);
        println!("\nResult: {}", result);
    } else {
        println!("\nInvalid arguments. Usage: power <base> <exponent>");
    }
}

/// Calculates the power of a number.
///
/// # Arguments
///
/// * `base` - The base number.
/// * `exponent` - The exponent.
///
/// # Returns
///
/// The result of `base` raised to the power of `exponent`.
fn power(base: f64, exponent: f64) -> f64 {
    let mut result = 1.0;
    for _ in 0..exponent as usize {
        result *= base;
    }
    result
}

/// Print memory details
fn check_memory() {
    println!("\nMemory Info");
    println!("Total GB: {}", MEMORY.get().unwrap().lock().total_mem_gigabytes());
    println!("Total MB: {}", MEMORY.get().unwrap().lock().total_mem_megabytes());
    println!("Total KB: {}", MEMORY.get().unwrap().lock().total_mem_kilobytes());
    println!("Total bytes: {}", MEMORY.get().unwrap().lock().total_memory);
    
    println!("Used GB: {}", MEMORY.get().unwrap().lock().total_used_mem_gigabytes());
    println!("Used MB: {}", MEMORY.get().unwrap().lock().total_used_mem_megabytes());
    println!("Used KB: {}", MEMORY.get().unwrap().lock().total_used_mem_kilobytes());
    println!("Used bytes: {}", MEMORY.get().unwrap().lock().used_memory);
    
}

/// Executes the "color" command.
///
/// # Arguments
///
/// * `args` - The arguments passed to the command.
fn change_color(args: &[&str]) {
    if args.is_empty() || args.len() != 1 {
        println!(
            "\nInvalid Arguments. Example Usage: color <red>   - Tip Use color help or color /?"
        );
    } else {
        let arg = args[0];

        if arg == "/?" || arg == "help" {
            println!(
                "\nExample Usage: color <red>\n\nAvailable Colors:\n
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
            white"
            );
            return;
        }
        if arg == "black" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_text_color(Color::Black);
        } else if arg == "blue" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_text_color(Color::Blue);
        } else if arg == "green" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_text_color(Color::Green);
        } else if arg == "cyan" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_text_color(Color::Cyan);
        } else if arg == "red" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_text_color(Color::Red);
        } else if arg == "magenta" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_text_color(Color::Magenta);
        } else if arg == "brown" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_text_color(Color::Brown);
        } else if arg == "lightgray" || arg == "lightgrey" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_text_color(Color::LightGray);
        } else if arg == "darkgray" || arg == "darkgrey" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_text_color(Color::DarkGray);
        } else if arg == "lightblue" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_text_color(Color::LightBlue);
        } else if arg == "lightgreen" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_text_color(Color::LightGreen);
        } else if arg == "lightcyan" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_text_color(Color::LightCyan);
        } else if arg == "lightred" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_text_color(Color::LightRed);
        } else if arg == "pink" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_text_color(Color::Pink);
        } else if arg == "yellow" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_text_color(Color::Yellow);
        } else if arg == "white" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_text_color(Color::White);
        } else {
            println!("\nInvalid Color: {}  - Tip Use color help or color /?", arg);
        }
    }
}

/// Executes the "color" command.
///
/// # Arguments
///
/// * `args` - The arguments passed to the command.
fn change_background_color(args: &[&str]) {
    if args.is_empty() || args.len() != 1 {
        println!(
            "\nInvalid Arguments. Example Usage: bgcolor <red>   - Tip Use bgcolor help or bgcolor /?"
        );
    } else {
        let arg = args[0];

        if arg == "/?" || arg == "help" {
            println!(
                "\nExample Usage: bgcolor <red>\n\nAvailable Colors:\n
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
            white"
            );
            return;
        }
        if arg == "black" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_background_color(Color::Black);
        } else if arg == "blue" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_background_color(Color::Blue);
        } else if arg == "green" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_background_color(Color::Green);
        } else if arg == "cyan" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_background_color(Color::Cyan);
        } else if arg == "red" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_background_color(Color::Red);
        } else if arg == "magenta" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_background_color(Color::Magenta);
        } else if arg == "brown" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_background_color(Color::Brown);
        } else if arg == "lightgray" || arg == "lightgrey" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_background_color(Color::LightGray);
        } else if arg == "darkgray" || arg == "darkgrey" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_background_color(Color::DarkGray);
        } else if arg == "lightblue" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_background_color(Color::LightBlue);
        } else if arg == "lightgreen" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_background_color(Color::LightGreen);
        } else if arg == "lightcyan" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_background_color(Color::LightCyan);
        } else if arg == "lightred" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_background_color(Color::LightRed);
        } else if arg == "pink" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_background_color(Color::Pink);
        } else if arg == "yellow" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_background_color(Color::Yellow);
        } else if arg == "white" {
            FRAMEBUFFER
                .get()
                .unwrap()
                .lock()
                .change_background_color(Color::White);
        } else {
            println!("\nInvalid Color: {}  - Tip Use bgcolor help or bgcolor /?", arg);
        }
    }
}

/// Executes the "echo" command.
///
/// # Arguments
///
/// * `s` - The arguments passed to the command.
fn echo(s: &[&str]) {
    println!("");
    for word in s {
        print!("{} ", word)
    }
    println!("");
}

/// Executes the "help" command.
fn help_command() {
    println!("\nAvailable Commands:\n");
    println!("hello");
    println!("add <num1> <num2>");
    println!("subtract <num1> <num2>");
    println!("multiply <num1> <num2>");
    println!("divide <num1> <num2>");
    println!("power <Base> <Exponent>");
    println!("mem");
    println!("color <color_name>");
    println!("bgcolor <color_name>");
    println!("echo <text>");
    println!("test");
    println!("clear");
    println!("rainbow");
    println!("help");
}
