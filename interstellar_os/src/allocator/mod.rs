//This file contains code for interstellar OS - https://github.com/interstellarfrog/interstellar
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

use core::alloc::Allocator;

use alloc::alloc::Global;
use x86_64::{
    structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags},
    VirtAddr,
};

mod fixed_size_block;

use fixed_size_block::FixedSizeBlockAllocator;

pub const HEAP_START: usize = 0x4444_4444_0000;
pub const HEAP_SIZE: usize = 16 * 1024 * 1024;

#[global_allocator]
pub static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(FixedSizeBlockAllocator::new());

pub fn init(
    physical_memory_offset: Option<u64>,
    memory_regions: &'static mut bootloader_api::info::MemoryRegions,
) {
    unsafe {
        crate::memory::init(physical_memory_offset.unwrap_or(0), memory_regions);
    }

    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    let mut binding = crate::memory::MAPPER.lock();
    let mapper = binding.as_mut().unwrap();
    let mut binding = crate::memory::FRAME_ALLOCATOR.lock();
    let frame_allocator = binding.as_mut().unwrap();
    for page in page_range {
        let frame = frame_allocator.allocate_frame().unwrap();
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper
                .map_to(page, frame, flags, frame_allocator)
                .unwrap()
                .flush()
        };
    }

    unsafe { ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE) };
}

pub struct Locked<T> {
    inner: spin::Mutex<T>,
}

impl<T> Locked<T> {
    pub const fn new(inner: T) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<T> {
        self.inner.lock()
    }
}

pub struct AlignedAlloc<const N: usize>;

unsafe impl<const N: usize> Allocator for AlignedAlloc<N> {
    fn allocate(
        &self,
        layout: core::alloc::Layout,
    ) -> Result<core::ptr::NonNull<[u8]>, core::alloc::AllocError> {
        Global.allocate(layout.align_to(N).unwrap())
    }

    unsafe fn deallocate(&self, ptr: core::ptr::NonNull<u8>, layout: core::alloc::Layout) {
        Global.deallocate(ptr, layout.align_to(N).unwrap())
    }
}
