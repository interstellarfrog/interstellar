use conquer_once::spin::OnceCell;
use core::{
    pin::Pin,
    task::{Context, Poll}
};
use crossbeam_queue::ArrayQueue;
use crate::console::handle_console;
use crate::vga_buffer::{Buffer, BUFFER_WIDTH, WRITER};
use crate::{print, println};
use futures_util::stream::{Stream, StreamExt};
use futures_util::task::AtomicWaker;
use pc_keyboard::{layouts, DecodedKey, HandleControl, KeyCode, KeyEvent, Keyboard, ScancodeSet1};
use alloc::string::String;

static WAKER: AtomicWaker = AtomicWaker::new();

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Uk105Key,
        HandleControl::Ignore,
    );

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            // If Enter Key Pressed
            if key_event == KeyEvent::new(KeyCode::Return, pc_keyboard::KeyState::Down) {

                let buffer = unsafe { &mut *(0xb8000 as *mut Buffer) };

                let writer = &WRITER;
                let row = writer.lock().return_row();

                let mut line = String::from("");

                // For Every Char In Line Add Char To List
                for i in 0..BUFFER_WIDTH {
                    let x = buffer.chars[row][i].read();
                    line.push(char::from(x.ascii_character));
                }

                // If The Line Contains A Terminal Start We Remove It And Send The Rest To The Console Module
                if line.contains("Neutron> ") {
                    line = line.replace("\n", "");
                    line = line.replace("Neutron> ", "");


                    handle_console(&line);
                };
                
                print!("\nNeutron> ")
            } else if key_event == KeyEvent::new(KeyCode::Backspace, pc_keyboard::KeyState::Down) {
                let writer = &WRITER;
                writer.lock().delete_char();

            } else {
                if let Some(key) = keyboard.process_keyevent(key_event) {
                    match key {
                        DecodedKey::Unicode(character) => print!("{}", character),
                        DecodedKey::RawKey(_key) => {/*print!("{:?}", _key)*/},
                    }
                }
            }
        }
    }
}

/// Called by the keyboard interrupt handler
///
/// Must not block or allocate.
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; droppong keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

pub struct ScancodeStream {
    _private: (), // Makes It So You Cannot Construct The Struct From Outside Of The Module
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new Should Only Be Called Once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        // Get Reference To Initialized Scancode Queue
        let queue = SCANCODE_QUEUE.try_get().expect("Not Initialized");

        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&cx.waker());
        // Get Next Element From Queue
        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}
