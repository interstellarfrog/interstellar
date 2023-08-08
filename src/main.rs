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

#![feature(custom_test_frameworks)]
#![test_runner(interstellar_os_test_runner::test_runner)]
#![reexport_test_harness_main = "test_main"]

use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::{env, process::Command};

fn main() {
    // Choose whether to start the UEFI or BIOS image
    let mut uefi = false; // UEFI still needs some more code

    let mut bios = false;

    // Choose whether to run GDB for debugging
    let mut gdb = false;

    // Convert the args into a collection for easier processing
    let args: Vec<String> = env::args().collect();

    // Start at index 1 (0 is command name)
    for arg in &args[1..] {
        match arg.as_str() {
            "--gdb" => gdb = true,
            "--uefi" => uefi = true,
            "--bios" => bios = true,
            _ => (),
        }
    }

    if !uefi && !bios {
        bios = true;
    }

    let uefi_path = PathBuf::from(env::var("UEFI_PATH").unwrap());
    let bios_path = PathBuf::from(env::var("BIOS_PATH").unwrap());
    let os_path = PathBuf::from(env::var("OS_PATH").unwrap());

    let mut qemu_cmd = Command::new("qemu-system-x86_64");

    if gdb {
        // If debugging using gdb, enable QEMU's built-in GDB server
        qemu_cmd.arg("-s");
        qemu_cmd.arg("-S");
        qemu_cmd.arg("-no-reboot");
    }

    println!("\nUEFI IMAGE Path: {}", uefi_path.display());
    println!("\nBIOS IMAGE Path: {}", bios_path.display());
    println!("\nKernel Path: {}", os_path.display());

    if uefi {
        bios = false; // Disable BIOS if running UEFI
        println!("\nRunning UEFI Kernel");
        qemu_cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        qemu_cmd
            .arg("-drive")
            .arg(format!("format=raw,file={}", uefi_path.display()));
    }
    if bios {
        println!("\nRunning BIOS Kernel");
        qemu_cmd
            .arg("-drive")
            .arg(format!("format=raw,file={}", bios_path.display()));
    }

    qemu_cmd.arg("-serial").arg("stdio"); // Redirect Serial To STDIO
    qemu_cmd.arg("-cpu").arg("max"); // Enables all features supported by the accelerator in the current host; Needed for RDSEED
    qemu_cmd.arg("-smp").arg("3");
    qemu_cmd.arg("-m").arg("1G,slots=3,maxmem=4G"); // Set Memory Size

    println!("\nStarting QEMU");

    let mut qemu_child = qemu_cmd.spawn().unwrap();

    if gdb {
        // If using gdb and in debug build mode, start GDB and connect to the QEMU GDB server
        thread::sleep(Duration::from_secs(3)); // Wait For QEMU To Start
        println!("Starting GDB");
        start_gdb(os_path);
    }

    // Wait for QEMU to exit
    qemu_child.wait().unwrap();
}

#[cfg(target_os = "windows")]
fn start_gdb(os_path: PathBuf) {
    let mut cmd_cmd = Command::new("cmd.exe");

    cmd_cmd.arg("/k");
    cmd_cmd.arg("start");
    cmd_cmd.arg("cmd.exe");
    cmd_cmd.arg("/k");
    cmd_cmd.arg("gdb.exe");
    cmd_cmd.arg("-x").arg("./gdb/gdbinit/.gdbinit");
    cmd_cmd.arg("-x").arg("./gdb/gdbinit/target.gdb");
    cmd_cmd.arg("-ex").arg(format!(
        "add-symbol-file {} -o 0xffff800000029a30",
        os_path.display()
    ));
    cmd_cmd.arg("-ex").arg("break *0x8000029a30");
    cmd_cmd.arg("-ex").arg("break panic");
    cmd_cmd.arg("-x").arg("./gdb/gdbinit/commands.gdb");

    let mut cmd_child = cmd_cmd
        .current_dir(env::current_dir().unwrap())
        .spawn()
        .expect("Failed to start GDB");

    cmd_child.wait().unwrap();
}

#[cfg(target_os = "linux")]
fn start_gdb(os_path: PathBuf) {
    let mut cmd_cmd = Command::new("gnome-terminal");

    cmd_cmd.arg("--").arg("gdb.exe");
    cmd_cmd.arg("-x").arg("./gdb/gdbinit/.gdbinit");
    cmd_cmd.arg("-x").arg("./gdb/gdbinit/target.gdb");
    cmd_cmd.arg("-ex").arg(format!(
        "add-symbol-file {} -o 0xffff800000029a30",
        os_path.display()
    ));
    cmd_cmd.arg("-ex").arg("break *0x8000029a30");
    cmd_cmd.arg("-ex").arg("break panic");
    cmd_cmd.arg("-x").arg("./gdb/gdbinit/commands.gdb");

    let mut cmd_child = cmd_cmd
        .current_dir(env::current_dir().unwrap())
        .spawn()
        .expect("Failed to start GDB");

    cmd_child.wait().unwrap();
}

#[cfg(target_os = "macos")]
fn start_gdb(_os_path: PathBuf) {
    todo!();
}
