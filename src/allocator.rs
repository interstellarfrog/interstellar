use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

use linked_list_allocator::LockedHeap;

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

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

    unsafe{ ALLOCATOR.lock().init(HEAP_START as *mut u8, HEAP_SIZE); }

    Ok(())
}


pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called")
    }
}