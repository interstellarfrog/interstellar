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

pub mod executor;
pub mod keyboard;
pub mod mouse;
pub mod simple_executor;
pub mod console_handler;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicU64, Ordering};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

pub struct Task {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }
    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}


/// Async Stream For Executor
macro_rules! stream_processor_task {
    ($stream_type:ty, $queue_size:literal) => {
        use core::{
            pin::Pin,
            task::{Context, Poll},
        };

        use conquer_once::spin::OnceCell;
        use crossbeam_queue::ArrayQueue;
        use futures_util::{task::AtomicWaker, Stream, StreamExt};

        static QUEUE: OnceCell<ArrayQueue<$stream_type>> = OnceCell::uninit();
        static WAKER: AtomicWaker = AtomicWaker::new();

        pub fn write(scancode: $stream_type) {
            if let Ok(queue) = QUEUE.try_get() {
                if queue.push(scancode).is_err() {

                } else {
                    WAKER.wake();
                }
            } else {

            }
        }

        struct TaskStream {
            _private: (),
        }
        impl TaskStream {
            fn new() -> Self {
                QUEUE
                    .try_init_once(|| ArrayQueue::new($queue_size))
                    .expect("queue failed to init");
                Self { _private: () }
            }
        }
        impl Stream for TaskStream {
            type Item = $stream_type;

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<$stream_type>> {
                let queue = QUEUE.try_get().expect("queue should be initialized by now");

                // fast
                if let Some(sc) = queue.pop() {
                    return Poll::Ready(Some(sc));
                }

                WAKER.register(cx.waker());
                match queue.pop() {
                    Some(sc) => {
                        WAKER.take();
                        Poll::Ready(Some(sc))
                    }
                    None => Poll::Pending,
                }
            }
        }
    };
}
pub(crate) use stream_processor_task;