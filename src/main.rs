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
use std::path::PathBuf;
use std::{env, process::Command};
use std::thread;
use std::time::Duration;

fn main() {
    // Read environment variables that were set in the build script
    let uefi_path = env!("UEFI_PATH");
    let bios_path = env!("BIOS_PATH");

    // Choose whether to start the UEFI or BIOS image
    let uefi = false;

    // Choose whether to run GDB for debugging
    let mut gdb = false;

    let mut k_path = PathBuf::from("");

    if !cfg!(debug_assertions) { // Not In Debug Mode
        gdb = false;
    } else {// In Debug Mode
        let kernel_path = env!("KERNEL_PATH");
        k_path = PathBuf::from(kernel_path);
    }
    

    let mut qemu_cmd = Command::new("qemu-system-x86_64");

    if gdb {
        // If debugging using gdb, enable QEMU's built-in GDB server
        qemu_cmd.arg("-s");
        qemu_cmd.arg("-S");
    }

    if uefi {
        println!("\nUEFI IMAGE Path: {}", uefi_path);
        println!("\nRunning UEFI Kernel");
        qemu_cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
        qemu_cmd.arg("-drive").arg(format!("format=raw,file={}", uefi_path));
    } else {
        println!("\nBIOS IMAGE Path: {}", bios_path);
        println!("\nRunning BIOS Kernel");
        qemu_cmd.arg("-drive").arg(format!("format=raw,file={}", bios_path));
    }

    qemu_cmd.arg("-serial").arg("stdio"); // Redirect Serial To STDIO
    qemu_cmd.arg("-device").arg("virtio-mouse"); // Emulate Mouse
    qemu_cmd.arg("-device").arg("virtio-keyboard"); // Emulate Keyboard

    println!("\nStarting QEMU");

    let mut qemu_child = qemu_cmd.spawn().unwrap();

    if gdb {
        // If using gdb and in debug build mode, start GDB and connect to the QEMU GDB server
        thread::sleep(Duration::from_secs(3)); // Wait For QEMU To Start
        println!("Starting GDB");
        start_gdb(k_path);
    }

    // Wait for QEMU to exit
    qemu_child.wait().unwrap();
    
}



#[cfg(target_os = "windows")]
fn start_gdb(k_path: PathBuf) {
    let current_dir = env::current_dir().expect("Failed to get current directory.");
    let gdb_dir = current_dir.join("gdb-windows").join("bin");
    env::set_current_dir(&gdb_dir).expect("Failed to change directory.");

    let mut cmd_cmd = Command::new("cmd.exe");
    cmd_cmd.arg("/k");
    cmd_cmd.arg("start");
    cmd_cmd.arg("cmd.exe");
    cmd_cmd.arg("/k");
    cmd_cmd.arg("gdb.exe");
    cmd_cmd.arg("-x").arg("gdbinit/.gdbinit");
    cmd_cmd.arg("-x").arg("gdbinit/target.gdb");
    cmd_cmd.arg("-ex").arg("add-symbol-file ../../target/debug/build/bootloader-61cf37ce9e863966/out/bin/bootloader-x86_64-bios-stage-4");
    cmd_cmd.arg("-ex").arg(format!("add-symbol-file {}", k_path.to_string_lossy().replace("\\", "/")));
    cmd_cmd.arg("-x").arg("gdbinit/commands.gdb");

    let mut cmd_child = cmd_cmd
        .current_dir(env::current_dir().unwrap())
        .spawn()
        .expect("Failed to start GDB");

    cmd_child.wait().unwrap();

}

#[cfg(target_os = "linux")]
fn start_gdb(k_path: PathBuf) {
    let current_dir = env::current_dir().expect("Failed to get current directory.");
    let gdb_dir = current_dir.join("gdb-linux").join("bin");
    env::set_current_dir(&gdb_dir).expect("Failed to change directory.");

    let mut cmd_cmd = Command::new("gnome-terminal");
    cmd_cmd.arg("--").arg("gdb");
    cmd_cmd.arg("-x").arg("gdbinit/.gdbinit");
    cmd_cmd.arg("-x").arg("gdbinit/target.gdb");
    cmd_cmd.arg("-ex").arg("add-symbol-file ../../target/debug/build/bootloader-61cf37ce9e863966/out/bin/bootloader-x86_64-bios-stage-4");
    cmd_cmd.arg("-ex").arg(format!("add-symbol-file {}", k_path.to_string_lossy().replace("\\", "/")));
    cmd_cmd.arg("-x").arg("gdbinit/commands.gdb");

    let mut cmd_child = cmd_cmd
        .current_dir(env::current_dir().unwrap())
        .spawn()
        .expect("Failed to start GDB");

    cmd_child.wait().unwrap();

}