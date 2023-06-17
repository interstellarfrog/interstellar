# Description

This is my OS that I am making while learning about Rust x86_64 systems programming.

### Aims

My current aims are to implement a user mode with a file system, GUI, some games, maybe port the Rust compiler and Python, and try to implement some Linux system calls and run some Linux-made apps.

### What can it do?

It might not seem like much from inside the OS, but there is a lot of code to make it work.

Currently, it has a PS/2 keyboard and mouse driver as input, but most VMs can translate your USB keyboard into PS/2 automatically.

Inside the OS, you can see your mouse cursor and use the Neutron Kernel Shell.

# Running the OS

From my knowledge, running the OS would require a full VGA setup with PS/2 mouse and keyboard, and even then it would reset every boot. Additionally, you would need to keep the USB in the PC if it would even work at all. However, don't worry, we can use a virtual machine.


## QEMU

The easiest way to run this is the QEMU virtual machine.

QEMU Downloads:

[Windows-64 Bit](https://qemu.weilnetz.de/w64/)

[Windows-32 Bit](https://qemu.weilnetz.de/w32/)

[Linux](https://www.qemu.org/download/#linux)

[MacOS](https://www.qemu.org/download/#macos)

I recommend you add it to your environment path variable, then run the .img file using QEMU with this command:

`qemu-system-x86_64 -drive format=raw,file=interstellar_os.img`

![qemu1.png](images/qemu1.jpg)

![qemu2.png](images/qemu2.png)

## Virtual box

VirtualBox takes some more setup than QEMU.

1. Launch VirtualBox and make a new empty unknown 64-bit machine.
![create_os_vbox.png](images/create_os_vbox.png)

2. Allocate the desired number of CPUs and a reasonable amount of memory for the machine. Then click on "Use an existing virtual hard disk file," add the .VDI file, and select it.
![create_os_vbox2.png](images/create_os_vbox2.png)

3. Then run it!
![vbox1.jpg](images/vbox1.jpg)

![vbox2.jpg](images/vbox2.png)

# Manually building

Manually building is easy. Just clone the repository, make sure you're using Rust Nightly Compiler with Rustup, and run this command for the bootloader to work:

`rustup component add llvm-tools-preview`

Make sure you have QEMU installed and added to your environment path variable. Then, from the root folder, run:

`cargo build && cargo run`

Note Building and running in release mode will make debug_println not print to the screen and should optimize the code away

`cargo build -r && cargo run -r`

I have also added an upgraded version of GNU Debugger for Windows from here:  [https://github.com/ssbssa/gdb](https://github.com/ssbssa/gdb) (Credit to ssbssa). It can be run automatically when in debug build mode by changing the GDB variable to `true` in `main.rs`.

For Linux Gnome-terminal users, add your GDB.exe into the gdb-linux bin folder. Adding it as a path variable may also work.

If you want to add more automatic commands on launch, edit the files in `gdb/bin/gdbinit.`

![GDB1.png](images/GDB1.png)
![GDB2.png](images/GDB2.png)


# Want to learn how to code your own OS in rust?

If you're interested in coding your own OS in Rust, check out the following resources:
[https://os.phil-opp.com/](https://os.phil-opp.com/)
[https://wiki.osdev.org/Expanded_Main_Page](https://wiki.osdev.org/Expanded_Main_Page)

Feel free to explore these resources to deepen your understanding of OS development in Rust.