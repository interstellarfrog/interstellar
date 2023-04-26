use super::align_up;
use core::mem;
use super::Locked;
use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr;



unsafe impl GlobalAlloc for Locked<LinkedListAllocator> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
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

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // Perform Layout Adjustments
        let (size, _) = LinkedListAllocator::size_align(layout);

        self.lock().add_free_region(ptr as usize, size)
    }
}



pub struct LinkedListAllocator {
    head: ListNode,
}

impl LinkedListAllocator {
    pub const fn new() -> Self {
        Self { head: ListNode::new(0), }
    }


    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.add_free_region(heap_start, heap_size);
    }

    unsafe fn add_free_region(&mut self, addr: usize, size: usize) {
        // Check If The Freed Region Is Capable Of Holding ListNode
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
        assert!(size >= mem::size_of::<ListNode>());
        
        // Create A New List Node And Append It At The Start Of The List
        let mut node = ListNode::new(size);
        node.next = self.head.next.take(); // 
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
            if let Ok(alloc_start) = Self::alloc_from_region(&region, size, align) {
                // Region Suitable For Allocation = Remove From Node List
                let next = region.next.take();
                let ret = Some((current.next.take().unwrap(), alloc_start));
                current.next = next;
                return ret;
            } else {
                // Region Not Suitable = Contine With Next Region
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

    /// Adjust The Given Layout So That The Resulting Allocated Memory
    /// Region Is Also Capable Of Storing A `ListNode`
    /// Returns The Adjusted Size And Alignment As A (Size, Align) Tuple
    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout.align_to(mem::align_of::<ListNode>()).expect("Adjusting Alignment Failed").pad_to_align();
        let size = layout.size().max(mem::size_of::<ListNode>());
        (size, layout.align())
    }

}

struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}

impl ListNode {
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }
    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }
    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
     }
}

