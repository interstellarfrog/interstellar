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

use bootloader_api::info::{MemoryRegion, MemoryRegionKind, MemoryRegions};
use conquer_once::spin::OnceCell;
use lazy_static::lazy_static;
use spin::Mutex;
use spinning_top::Spinlock;
use x86_64::{
    structures::paging::{
        page::PageRange, FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable,
        PageTableFlags, PhysFrame, Size4KiB, Translate,
    },
    PhysAddr, VirtAddr,
};

use os_units::{Bytes, NumOfPages};

lazy_static! {
    pub static ref MAPPER: Mutex<Option<OffsetPageTable<'static>>> = Mutex::new(None);
    pub static ref FRAME_ALLOCATOR: Mutex<Option<BootInfoFrameAllocator>> = Mutex::new(None);
    pub static ref MEMORY: OnceCell<Spinlock<Memory>> = OnceCell::uninit();
}

/// Initialize the Frame allocator and Mapper
///
/// # Safety
///
/// This function is unsafe because the caller must guarantee that the passed `physical_memory_offset` is valid
pub unsafe fn init(physical_memory_offset: u64, memory_regions: &'static mut MemoryRegions) {
    let level_4_table = active_level_4_table(physical_memory_offset);
    let table = OffsetPageTable::new(level_4_table, VirtAddr::new(physical_memory_offset));

    let _ = MAPPER.lock().insert(table);

    let mut total_memory = 0;

    for m in memory_regions.iter() {
        // Pretty sure this is how we calculate aproximate total memory
        if m.kind == MemoryRegionKind::Usable || m.kind == MemoryRegionKind::Bootloader {
            total_memory += m.end - m.start;
        }
    }

    MEMORY.init_once(|| {
        Spinlock::new(Memory {
            total_memory,
            used_memory: 0,
        })
    });

    let frame_allocator = unsafe { BootInfoFrameAllocator::init(memory_regions) };

    let _ = FRAME_ALLOCATOR.lock().insert(frame_allocator);
}

unsafe fn active_level_4_table(physical_memory_offset: u64) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = VirtAddr::new(physical_memory_offset + phys.as_u64());
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

pub struct Memory {
    pub total_memory: u64,
    pub used_memory: u64,
}

impl Memory {
    pub fn total_mem_kilobytes(&self) -> f32 {
        let size_in_bytes = self.total_memory;
        size_in_bytes as f32 / 1024.0
    }

    pub fn total_used_mem_kilobytes(&self) -> f32 {
        let size_in_bytes = self.used_memory;
        size_in_bytes as f32 / 1024.0
    }

    pub fn total_mem_megabytes(&self) -> f32 {
        let size_in_bytes = self.total_memory;
        let size_in_kilobytes = size_in_bytes as f32 / 1024.0;
        size_in_kilobytes / 1024.0
    }

    pub fn total_used_mem_megabytes(&self) -> f32 {
        let size_in_bytes = self.used_memory;
        let size_in_kilobytes = size_in_bytes as f32 / 1024.0;
        size_in_kilobytes / 1024.0
    }

    pub fn total_mem_gigabytes(&self) -> f32 {
        let size_in_bytes = self.total_memory;
        let size_in_kilobytes = size_in_bytes as f32 / 1024.0;
        let size_in_megabytes = size_in_kilobytes / 1024.0;
        size_in_megabytes / 1024.0
    }

    pub fn total_used_mem_gigabytes(&self) -> f32 {
        let size_in_bytes = self.used_memory;
        let size_in_kilobytes = size_in_bytes as f32 / 1024.0;
        let size_in_megabytes = size_in_kilobytes / 1024.0;
        size_in_megabytes / 1024.0
    }

    pub fn add_to_used_mem(&mut self, amount: u64) {
        self.used_memory += amount;
    }

    pub fn takeaway_from_used_mem(&mut self, amount: u64) {
        self.used_memory -= amount;
    }
}

/// A [FrameAllocator] that returns usable frames from the bootloadeer's memory map
pub struct BootInfoFrameAllocator {
    memory_regions: &'static mut [MemoryRegion],
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a [FrameAllocator] from the passed memory map.
    ///
    /// # Safety
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_regions: &'static mut MemoryRegions) -> Self {
        BootInfoFrameAllocator {
            memory_regions,
            next: 0,
        }
    }
    /// Returns an iterator of usable frames
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> + '_ {
        let usable_regions = self
            .memory_regions
            .iter()
            .filter(|r| r.kind == MemoryRegionKind::Usable);
        // map each region to its address range
        let addr_ranges = usable_regions.map(|r| r.start..r.end);
        // transform to an iterator of frame start addresses
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // create `PhysFrame` types from the start addresses
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}

fn search_free_addr_from(num_pages: NumOfPages<Size4KiB>, region: PageRange) -> Option<VirtAddr> {
    let mut cnt = 0;
    let mut start = None;
    for page in region {
        let addr = page.start_address();
        if available(addr) {
            if start.is_none() {
                start = Some(addr);
            }

            cnt += 1;

            if cnt >= num_pages.as_usize() {
                return start;
            }
        } else {
            cnt = 0;
            start = None;
        }
    }

    None
}

fn available(addr: VirtAddr) -> bool {
    let mut binding = MAPPER.lock();
    let mapper = binding.as_mut().unwrap();

    mapper.translate_addr(addr).is_none() && !addr.is_null()
}

pub fn map_pages_from(start: PhysAddr, object_size: usize, region: PageRange) -> VirtAddr {
    let start_frame_addr = start.align_down(Size4KiB::SIZE);
    let end_frame_addr = (start + object_size).align_down(Size4KiB::SIZE);

    let num_pages =
        Bytes::new((end_frame_addr - start_frame_addr) as usize + 1).as_num_of_pages::<Size4KiB>();

    let virt = search_free_addr_from(num_pages, region).expect("error searching for free addr");

    let mut mapper = MAPPER.lock();
    let mapper = mapper.as_mut().unwrap();

    let mut frame_allocator = FRAME_ALLOCATOR.lock();
    let frame_allocator = frame_allocator.as_mut().unwrap();

    for i in 0..num_pages.as_usize() {
        let page = Page::<Size4KiB>::containing_address(virt + Size4KiB::SIZE * i as u64);
        let frame = PhysFrame::containing_address(start_frame_addr + Size4KiB::SIZE * i as u64);
        let flag =
            PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE;

        unsafe {
            mapper
                .map_to(page, frame, flag, frame_allocator)
                .unwrap()
                .flush();
        }
    }

    let page_offset = start.as_u64() % Size4KiB::SIZE;

    virt + page_offset
}

pub fn map_address(phys: PhysAddr, size: usize) -> VirtAddr {
    map_pages_from(
        phys,
        size,
        PageRange {
            start: Page::from_start_address(VirtAddr::new(0xFFFF_8000_0000_0000)).unwrap(),
            end: Page::containing_address(VirtAddr::new(0xFFFF_FFFF_FFFF_FFFF)),
        },
    )
}
