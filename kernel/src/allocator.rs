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

use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};
use fixed_size_block::FixedSizeBlockAllocator;

pub mod bump;
pub mod linked_list;
pub mod fixed_size_block;

#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB


pub fn init_heap( // Takes In Mapper And Frame Allocator
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64); // Get Heap Address
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start); // Get Heap Page From Address
        let heap_end_page = Page::containing_address(heap_end); // Get Heap End From Address
        Page::range_inclusive(heap_start_page, heap_end_page) // Creates A Range From Given Heap Pages
    };

    for page in page_range { // For Each Page In Range
        let frame = frame_allocator
            .allocate_frame() // Allocate Frame For Page
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE; // Add Present And Writable Flags
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush() // Map The Virtual Heap Address's To Real Ones
        };
    }

    unsafe{ ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE); }

    Ok(())
}




// A Wrapper Around spin::Mutex To Permit Trait Implementations
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

// Align Address To Alignment
// Align Needs To Be A Power Of 2
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1) // Bitwise Operations 
}
// An Easier To Understand Version
/*
fn align_up(addr: usize, align: usize) -> usize {
    let remainder = addr % align;
    if remainder == 0 {
        addr // addr already aligned
    } else {
        addr - remainder + align
    }
}
 */