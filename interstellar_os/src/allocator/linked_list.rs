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

use super::align_up;
use super::Locked;
use alloc::alloc::{GlobalAlloc, Layout};
use core::mem;
use core::ptr;

unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    /// Allocates a memory block of the given layout.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it performs low-level memory allocation operations.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        crate::memory::MEMORY.get().unwrap().force_unlock();
        crate::memory::MEMORY.get().unwrap().lock().takeaway_from_used_mem(layout.size().try_into().unwrap());
        // Perform Layout Adjustments
        let (size, align) = LinkedListAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.find_region(size, align) {
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let excess_size = region.end_addr() - alloc_end;
            if excess_size > 0 {
                allocator.add_free_region(alloc_end, excess_size);
            }
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    /// Deallocates the memory block pointed to by `ptr` with the given layout.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it performs low-level memory deallocation operations.
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        crate::memory::MEMORY.get().unwrap().force_unlock();
        crate::memory::MEMORY.get().unwrap().lock().takeaway_from_used_mem(layout.size().try_into().unwrap());
        // Perform Layout Adjustments
        let (size, _) = LinkedListAllocator::size_align(layout);

        self.lock().add_free_region(ptr as usize, size)
    }
}

/// A linked list memory allocator.
pub struct LinkedListAllocator {
    head: ListNode,
}

impl LinkedListAllocator {
    /// Creates a new instance of `LinkedListAllocator`.
    pub const fn new() -> Self {
        Self {
            head: ListNode::new(0),
        }
    }

    /// Initializes the allocator with the given heap start and size.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it performs low-level memory operations.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        // Check If The Freed Region Is Capable Of Holding ListNode
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
        assert!(size >= mem::size_of::<ListNode>());

        // Create A New List Node And Append It At The Start Of The List
        let mut node = ListNode::new(size);
        node.next = self.head.next.take();
        let node_ptr = addr as *mut ListNode;
        node_ptr.write(node);
        self.head.next = Some(&mut *node_ptr)
    }

    // Looks For Free Region With Given Size And Alignment And Removes It From The List
    // Returns Tuple Of The List Node And The Start Address Of The Allocation
    fn find_region(&mut self, size: usize, align: usize) -> Option<(&'static mut ListNode, usize)> {
        // Reference To Current List Node, Updated For Each Iteration
        let mut current = &mut self.head;

        // Look For Large Enough Memory Area In The Linked List
        while let Some(ref mut region) = current.next {
            if let Ok(alloc_start) = Self::alloc_from_region(region, size, align) {
                // Region Suitable For Allocation = Remove From Node List
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;
            } else {
                // Region Not Suitable = Continue With Next Region
                current = current.next.as_mut().unwrap();
            }
        }

        // No Suitable Region Found
        None
    }

    fn alloc_from_region(region: &ListNode, size: usize, align: usize) -> Result<usize, ()> {
        let alloc_start = align_up(region.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size).ok_or(())?;

        if alloc_end > region.end_addr() {
            // Region Too Small
            return Err(());
        } else {
            let excessive_size = region.end_addr() - alloc_end;
            if excessive_size > 0 && excessive_size < mem::size_of::<ListNode>() {
                // Rest Of Region Too Small To Hold A ListNode
                // Required Because The Allocation Splits The Region In A Used And A Free Part
                return Err(());
            }
        }

        Ok(alloc_start)
    }

    /// Adjusts the given layout so that the resulting allocated memory
    /// region is also capable of storing a `ListNode`.
    ///
    /// Returns the adjusted size and alignment as a `(size, align)` tuple.
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<ListNode>())
            .expect("Adjusting Alignment Failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }
}

/// A node in the linked list used by `LinkedListAllocator`.
struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    /// Creates a new instance of `ListNode` with the given size.
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }

    /// Returns the start address of the node.
    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    /// Returns the end address of the node.
    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}
