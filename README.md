# interstellar_os
This Is My OS That I Am Making Whilst Learning About Rust x86_64 Systems Programing

## Manually Building
As Far As I Know All You Need To Do To Build This Manually Is Run These Commands: rustup toolchain install nightly && rustup override set nightly && cargo install bootimage && rustup component add llvm-tools-preview

This Sets Your Rust Version To Nightly Which Is A Version With The Latest Updates We Need This For Some Experimental Features And It Also Downloads Some Tools That We Need

I Am Also Making A Powershell Tool To Set Up The Dev Environment Automatically 

# Running The OS
Currently This Can Only Be Run On VGA Hardware So Unless If You Have A VGA Monitor Running This On Real Hardware Will Not Work So Instead We Run This Using The QUEMU Virtual Machine

QUEMU Downloads

[Windows-64 Bit](https://qemu.weilnetz.de/w64/)

[Windows-32 Bit](https://qemu.weilnetz.de/w32/)

[Linux](https://www.qemu.org/download/#linux)

[MacOS](https://www.qemu.org/download/#macos)

You Can Run The .bin File Using QEMU With This Command: qemu-system-x86_64 -drive format=raw,file=bootimage-interstellar_os.bin

For Windows You Can Add The QUEMU Folder To Your Environmental Path Variable So You Can Access It Anywhere Or Just Drag Your .bin File Into The QUEMU Folder


## Where I Am Learning From
For The Start Of This I Am Using The Blog_OS Tutorial [https://os.phil-opp.com/](https://os.phil-opp.com/)