# Requirements

- Write all code in rust - I am trying to keep the github code usage at 100% for rust, for assembly use `core::arch::asm!` or `core::arch::global_asm!`

- Operating systems need to be fast so optimize your code as much as possible

- If you are implementing something for a specific system component make sure it is checked that this component is available before using this code and if possible conditionally use an alternative for this component if it is not available but still required

- At least run `cargo test -- --uefi` when finished if you changed or added any code

- If you think a test may be required for the code you added feel free to add one (just copy the basic_boot test and edit it)

- Some parts of the OS run in "parrallel" for example interrupt handlers so if a struct is needed by an interrupt handler or something else that runs in "parrallel" make sure you add locking mechanisms for example:

```Rust
pub static FRAMEBUFFER: conquer_once::spin::OnceCell<spinning_top::spinlock::Spinlock<FrameBufferWriter>> = OnceCell::uninit();

fn init frame_buffer() {
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

}


```


- Do not copy large amounts of code from license compatible resources - just feels like stealing

- Make sure to run cargo clippy and resolve all warnings

- Make sure to run cargo fmt --all

- Comment - Some OS concepts can be complex so use comments (I would rather have code comments on every line than no comments at all)

- Comment - If your creating functions or structs or anything you can add doc comments for "/// this is what this thing does", do so
