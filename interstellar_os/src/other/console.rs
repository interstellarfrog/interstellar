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
use crate::{
    drivers::screen::framebuffer::{Color, FRAMEBUFFER},
    memory::MEMORY,
    print, println,
};
use alloc::format;
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
        .trace("Handling console command", file!(), line!());

    if !line.is_empty() {
        print!("\n");
    }

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
                "time" => time_command(args),
                "color" => change_color(args),
                "bgcolor" => {
                    let clear = change_background_color(args);
                    return clear;
                }
                "colour" => change_color(args),
                "echo" => echo(args),
                "help" => help_command(),
                "test" => {
                    println!("Initializing Tests");
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
                "" => {
                    return false;
                }
                "stack_overflow" => {
                    stack_overflow();
                }
                _ => {
                    LOGGER
                        .get()
                        .unwrap()
                        .lock()
                        .error(&format!("UNKNOWN COMMAND - <{}>", command));
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
    println!("Hello, World!");
    if !args.is_empty() {
        println!("Arguments: {:?}", args);
    }
}

/// Executes the "add" command.
///
/// # Arguments
///
/// * `args` - The arguments passed to the command.
fn add_command(args: &[&str]) {
    if args.len() != 2 {
        LOGGER
            .get()
            .unwrap()
            .lock()
            .error("Invalid number of arguments. Usage: add <num1> <num2>");
    } else if let (Ok(num1), Ok(num2)) = (args[0].parse::<i128>(), args[1].parse::<i128>()) {
        let result = num1 + num2;
        println!("Sum: {}", result);
    } else {
        LOGGER
            .get()
            .unwrap()
            .lock()
            .error("Invalid arguments. Usage: add <num1> <num2>");
    }
}

/// Executes the "subtract" command.
///
/// # Arguments
///
/// * `args` - The arguments passed to the command.
fn subtract_command(args: &[&str]) {
    if args.len() != 2 {
        LOGGER
            .get()
            .unwrap()
            .lock()
            .error("Invalid number of arguments. Usage: subtract <num1> <num2>");
    } else if let (Ok(num1), Ok(num2)) = (args[0].parse::<i128>(), args[1].parse::<i128>()) {
        let result = num1 - num2;
        println!("\nDifference: {}", result);
    } else {
        LOGGER
            .get()
            .unwrap()
            .lock()
            .error("Invalid arguments. Usage: subtract <num1> <num2>");
    }
}

/// Executes the "multiply" command.
///
/// # Arguments
///
/// * `args` - The arguments passed to the command.
fn multiply_command(args: &[&str]) {
    if args.len() != 2 {
        LOGGER
            .get()
            .unwrap()
            .lock()
            .error("Invalid number of arguments. Usage: multiply <num1> <num2>");
    } else if let (Ok(num1), Ok(num2)) = (args[0].parse::<i128>(), args[1].parse::<i128>()) {
        let result = num1 * num2;
        println!("\nProduct: {}", result);
    } else {
        LOGGER
            .get()
            .unwrap()
            .lock()
            .error("Invalid arguments. Usage: multiply <num1> <num2>");
    }
}

/// Executes the "divide" command.
///
/// # Arguments
///
/// * `args` - The arguments passed to the command.
fn divide_command(args: &[&str]) {
    if args.len() != 2 {
        LOGGER
            .get()
            .unwrap()
            .lock()
            .error("Invalid number of arguments. Usage: multiply <num1> <num2>");
    } else if let (Ok(num1), Ok(num2)) = (args[0].parse::<f64>(), args[1].parse::<f64>()) {
        if num2 == 0.0 {
            println!("\nCannot divide by zero.");
        } else {
            let result = num1 / num2;
            println!("\nQuotient: {}", result);
        }
    } else {
        LOGGER
            .get()
            .unwrap()
            .lock()
            .error("Invalid arguments. Usage: divide <num1> <num2>");
    }
}

/// Executes the "power" command.
///
/// # Arguments
///
/// * `args` - The arguments passed to the command.
fn power_command(args: &[&str]) {
    if args.len() != 2 {
        LOGGER
            .get()
            .unwrap()
            .lock()
            .error("Invalid number of arguments. Usage: power <base> <exponent>");
    } else if let (Ok(base), Ok(exponent)) = (args[0].parse::<f64>(), args[1].parse::<f64>()) {
        let result = power(base, exponent);
        println!("\nResult: {}", result);
    } else {
        LOGGER
            .get()
            .unwrap()
            .lock()
            .error("Invalid arguments. Usage: power <base> <exponent>");
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
    println!("Memory Info");
    println!(
        "Total GB: {}",
        MEMORY.get().unwrap().lock().total_mem_gigabytes()
    );
    println!(
        "Total MB: {}",
        MEMORY.get().unwrap().lock().total_mem_megabytes()
    );
    println!(
        "Total KB: {}",
        MEMORY.get().unwrap().lock().total_mem_kilobytes()
    );
    println!("Total bytes: {}", MEMORY.get().unwrap().lock().total_memory);

    println!(
        "Used GB: {}",
        MEMORY.get().unwrap().lock().total_used_mem_gigabytes()
    );
    println!(
        "Used MB: {}",
        MEMORY.get().unwrap().lock().total_used_mem_megabytes()
    );
    println!(
        "Used KB: {}",
        MEMORY.get().unwrap().lock().total_used_mem_kilobytes()
    );
    println!("Used bytes: {}", MEMORY.get().unwrap().lock().used_memory);
}

fn time_command(args: &[&str]) {
    if args.is_empty() || args.len() != 1 {
        LOGGER.get().unwrap().lock().error(
            "Invalid number of arguments. Usage: time <command> - Tip Use time help or time /?",
        );
    } else {
        let arg = args[0];

        if arg == "/?" || arg == "help" {
            println!(
                "Example Usage: time <boot>\n\nAvailable Time Commands:\n
            boot"
            );
        } else if arg == "boot" {
            let boot_time = unsafe { crate::time::GLOBAL_TIMER.get().unwrap().lock().elapsed() };

            if boot_time.as_secs() > 60 {
                if (boot_time.as_secs() / 60) > 60 {
                    println!("Time Since Boot: {} Hrs", (boot_time.as_secs() / 60) / 60);
                } else {
                    println!(
                        "Time Since Boot: {} mins {}s",
                        boot_time.as_secs() / 60,
                        boot_time.as_secs() - ((boot_time.as_secs() / 60) * 60)
                    );
                }
            } else {
                println!("Time Since Boot: {}s", boot_time.as_secs());
            }
        } else {
            LOGGER.get().unwrap().lock().error(
                "Invalid Arguments. Example Usage: time <boot> - Tip Use time help or time /?",
            );
        }
    }
}

/// Executes the "color" command.
///
/// # Arguments
///
/// * `args` - The arguments passed to the command.
fn change_color(args: &[&str]) {
    if args.is_empty() || args.len() != 1 {
        LOGGER.get().unwrap().lock().error(
            "Invalid number of Arguments. Usage: color <color> - Tip Use color help or color /?",
        );
    } else {
        let arg = args[0];

        if arg == "/?" || arg == "help" {
            println!(
                "Example Usage: color <red>\n\nAvailable Colors:\n
            - white
            - black
            - blue
            - green
            - cyan
            - red
            - magenta
            - brown
            - lightgray
            - darkgray
            - lightblue
            - lightgreen
            - lightcyan
            - lightred
            - pink
            - yellow
            - midnightblue
            - orange
            - lavender
            - teal
            - gold
            - silver
            - violet
            - coral
            - aqua"
            );
            return;
        }

        let framebuffer = FRAMEBUFFER.get().unwrap();
        let color = match arg {
            "black" => Color::Black,
            "blue" => Color::Blue,
            "green" => Color::Green,
            "cyan" => Color::Cyan,
            "red" => Color::Red,
            "magenta" => Color::Magenta,
            "brown" => Color::Brown,
            "lightgray" | "lightgrey" => Color::LightGray,
            "darkgray" | "darkgrey" => Color::DarkGray,
            "lightblue" => Color::LightBlue,
            "lightgreen" => Color::LightGreen,
            "lightcyan" => Color::LightCyan,
            "lightred" => Color::LightRed,
            "pink" => Color::Pink,
            "yellow" => Color::Yellow,
            "white" => Color::White,
            "midnightblue" => Color::MidnightBlue,
            "orange" => Color::Orange,
            "lavender" => Color::Lavender,
            "teal" => Color::Teal,
            "gold" => Color::Gold,
            "silver" => Color::Silver,
            "violet" => Color::Violet,
            "coral" => Color::Coral,
            "aqua" => Color::Aqua,
            _ => {
                LOGGER.get().unwrap().lock().error(&format!(
                    "Invalid Color: {} - Tip Use color help or color /?",
                    arg
                ));
                return;
            }
        };

        framebuffer.lock().change_text_color(color);
    }
}

/// Executes the "bgcolor" command.
///
/// # Arguments
///
/// * `args` - The arguments passed to the command.
fn change_background_color(args: &[&str]) -> bool {
    if args.is_empty() || args.len() != 1 {
        LOGGER.get().unwrap().lock().error("Invalid number of arguments. Usage: bgcolor <color> - Tip Use bgcolor help or bgcolor /?");
        false
    } else {
        let arg = args[0];

        if arg == "/?" || arg == "help" {
            println!(
                "Example Usage: bgcolor <red>\n\nAvailable Colors:\n
            - white
            - black
            - blue
            - green
            - cyan
            - red
            - magenta
            - brown
            - lightgray
            - darkgray
            - lightblue
            - lightgreen
            - lightcyan
            - lightred
            - pink
            - yellow
            - midnightblue
            - orange
            - lavender
            - teal
            - gold
            - silver
            - violet
            - coral
            - aqua"
            );
            return false;
        }

        let framebuffer = FRAMEBUFFER.get().unwrap();
        let color = match arg {
            "black" => Color::Black,
            "blue" => Color::Blue,
            "green" => Color::Green,
            "cyan" => Color::Cyan,
            "red" => Color::Red,
            "magenta" => Color::Magenta,
            "brown" => Color::Brown,
            "lightgray" | "lightgrey" => Color::LightGray,
            "darkgray" | "darkgrey" => Color::DarkGray,
            "lightblue" => Color::LightBlue,
            "lightgreen" => Color::LightGreen,
            "lightcyan" => Color::LightCyan,
            "lightred" => Color::LightRed,
            "pink" => Color::Pink,
            "yellow" => Color::Yellow,
            "white" => Color::White,
            "midnightblue" => Color::MidnightBlue,
            "orange" => Color::Orange,
            "lavender" => Color::Lavender,
            "teal" => Color::Teal,
            "gold" => Color::Gold,
            "silver" => Color::Silver,
            "violet" => Color::Violet,
            "coral" => Color::Coral,
            "aqua" => Color::Aqua,
            _ => {
                LOGGER.get().unwrap().lock().error(&format!(
                    "Invalid color: {} - Tip Use bgcolor help or bgcolor /?",
                    arg
                ));
                return false;
            }
        };

        framebuffer.lock().change_background_color(color);

        FRAMEBUFFER.get().unwrap().lock().clear();

        true
    }
}

/// Executes the "echo" command.
///
/// # Arguments
///
/// * `s` - The arguments passed to the command.
fn echo(s: &[&str]) {
    for word in s {
        print!("{} ", word)
    }
    println!("");
}

/// Executes the "help" command.
fn help_command() {
    println!("Available Commands:\n");
    println!("hello");
    println!("clear/cls");
    println!("add <num1> <num2>");
    println!("subtract <num1> <num2>");
    println!("multiply <num1> <num2>");
    println!("divide <num1> <num2>");
    println!("power <Base> <Exponent>");
    println!("color <color_name>");
    println!("bgcolor <color_name>");
    println!("echo <text>");
    println!("test");
    println!("mem");
    println!("stack_overflow");
    println!("help");
}
