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

use super::{align_up, Locked};
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;

/// A simple bump allocator that allocates memory linearly from a fixed heap.
pub struct BumpAllocator {
    heap_start: usize,
    heap_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    /// Creates a new instance of `BumpAllocator`.
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    /// Initializes the allocator with the given heap start address and size.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it initializes the allocator with raw memory addresses.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    /// Allocates a memory block of the given layout using the bump allocator.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it performs low-level memory allocation operations.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Lock to stop data races
        let mut bump = self.lock(); // Mutable reference to wrapped allocator type

        let alloc_start = align_up(bump.next, layout.align());

        let alloc_end = match alloc_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if alloc_end > bump.heap_end {
            ptr::null_mut() // Out of memory
        } else {
            // Allocate memory
            bump.next = alloc_end;
            bump.allocations += 1;
            alloc_start as *mut u8
        }
    }

    /// Deallocates the memory block with the given pointer and layout.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it performs low-level memory deallocation operations.
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut bump = self.lock(); // Mutable reference to wrapped allocator type

        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.heap_start;
        }
    }
}
