# Contributing

By contributing to this project, you agree that your contributions will be licensed under this projects current license and you agree to the terms and conditions in this projects license.

- If your not a big code writter you can still help by documenting code if you understand it or by cleaning up or writing documents

## Code Requirements

- If you make a new file make sure to add this GPL3 license header:

```License
//This file contains code for interstellar OS - <https://github.com/interstellarfrog/interstellar>
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
```

- Write all code possible in rust - I am trying to keep the github code usage close to 100% for rust, for assembly use `core::arch::asm!` or `core::arch::global_asm!`

- Operating systems need to be fast so optimize your code as much as possible if its a final implementation

- If you are implementing something for a specific system component make sure it is checked that this component is available before using this code and if possible conditionally use an alternative for this component if it is not available but still required

- At least run `cargo test -- --uefi` when finished if you changed or added any code

- If you think a test may be required for the code you added feel free to add one (just copy the basic_boot test and edit it)

- Some parts of the OS run in "total parrallel" (they execute single assembly instructions then switch to a whole different function) for example interrupt handlers (they get interrupted at any time when other instructions could be running) so if a struct is needed by an interrupt handler or something else that runs in "total parrallel" make sure you add locking mechanisms so a race condition does not occur and when using locks avoid deadlocks for example:

```Rust
/// A global FrameBufferWriter for the OS
pub static FRAMEBUFFER: conquer_once::spin::OnceCell<spinning_top::spinlock::Spinlock<FrameBufferWriter>> = OnceCell::uninit();

fn main() {

    FRAMEBUFFER.init_once(|| { // This only needs to be done once for the whole code
        if let Some(info) = BOOT_INFO.get().unwrap().lock().framebuffer_info {
            if let Some(buffer) = boot_info.framebuffer.as_mut() {
                spinning_top::Spinlock::new(FrameBufferWriter::new(buffer.buffer_mut(), info)) // Return the new FrameBufferWriter
            } else {
                panic!("BOOTLOADER NOT CONFIGURED TO SUPPORT FRAMEBUFFER");
            }
        } else {
            panic!("BOOTLOADER NOT CONFIGURED TO SUPPORT FRAMEBUFFER");
        }
    });

    // We can now use FRAMEBUFFER anywhere in the code as long as we use it in a safe way

    FRAMEBUFFER.get().unwrap().lock().do_something();

    FRAMEBUFFER.get().unwrap().lock().do_something_else();

    // Warning: doing stuff like this will cause a deadlock

    if FRAMEBUFFER.get().unwrap().lock().get_value1() == 1 && FRAMEBUFFER.get().unwrap().lock().get_value2() == 2 {} // Causes a deadlock as .lock() is being called twice in one line

    // version of this that works:

    let value1 = FRAMEBUFFER.get().unwrap().lock().get_value1();

    if value1 == 1 && FRAMEBUFFER.get().unwrap().lock().get_value2() == 2 {} // Works fine

    // For functions that can occur at any time - hardware called functions, call this before getting another lock on it:

    unsafe {FRAMEBUFFER.get().unwrap().force_unlock()};

    // Now you can use it

    FRAMEBUFFER.get().unwrap().lock().do_something();
  
}


```

- Do not copy large amounts of code from license compatible resources - just feels like stealing

- Make sure to run cargo clippy and resolve all warnings

- Make sure to run cargo fmt --all

- Comment - Some OS concepts can be complex so use comments (I would rather have code comments on every line than no comments at all)

- Comment - If your creating functions or structs or anything you can add doc comments for "/// this is what this thing does", add them if its an important part of code
