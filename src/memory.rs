use x86_64::{ structures::paging::{PageTable, OffsetPageTable, Mapper, Size4KiB, FrameAllocator, Page, PhysFrame, PageTableFlags as Flags}, VirtAddr, registers::control::Cr3, PhysAddr};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }
    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // get usable regions from memory map
        let regions = self.memory_map.iter();
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr()..r.range.end_addr());
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

// Returns A Mapper To Map Virt Addr To Phys Addr
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> { // Never Dies
    let level_4_table = active_level_4_table(physical_memory_offset); //Get level 4 Page Table Reference
    OffsetPageTable::new(level_4_table, physical_memory_offset) // Return New Offset Page Table
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level_4_table_frame, _) = Cr3::read(); // Read Physical Frame Of The Active Level 4 Page Table
    let phys = level_4_table_frame.start_address(); // Get The Start Address
    let virt = physical_memory_offset + phys.as_u64(); // Add The Physical Memory Offset
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr(); // Create Raw Mutable Pointer
    &mut *page_table_ptr // Return Mutable Reference To The Raw Pointer
}


pub struct EmptyFrameAllocator;
// Allocates Frames If Needed By map_to
unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> { // Creates No New Frames
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}


pub fn create_example_mapping(page: Page, mapper: &mut OffsetPageTable, frame_allocator: &mut impl FrameAllocator<Size4KiB>) {
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe { // For Testing
        // FIX ME: UNSAFE
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("Map_To Failed").flush(); // If It Fails To Create A Mapping Flush The TLB
}