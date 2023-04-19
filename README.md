# interstellar_os
This Is My OS That I Am Making Whilst Learning About Rust Systems Programing

I Am Also Making A Powershell Tool To Set Up The Dev Environment Automatically 

As Far As I Know All You Need To Do To Build This Manually Is Run These Commands: rustup toolchain install nightly && rustup override set nightly && cargo install bootimage && rustup component add llvm-tools-preview

You Can Run The .BIN File Using QEMU With This Command: qemu-system-x86_64 -drive format=raw,file=bootimage-interstellar_os.bin