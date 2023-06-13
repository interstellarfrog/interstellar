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

use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

/// The size of the stack used for interrupt stack tables (IST) in bytes.
const STACK_SIZE: usize = 4096 * 5;

/// The index of the IST used for double faults.
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

/// The index of the IST used for page faults.
pub const PAGE_FAULT_IST_INDEX: u16 = 1;

/// The index of the IST used for general protection faults.
pub const GENERAL_PROTECTION_FAULT_IST_INDEX: u16 = 2;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // Set the stack pointer for privilege level 0 (kernel stack).
        tss.privilege_stack_table[0] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            VirtAddr::from_ptr(unsafe { &STACK }) + STACK_SIZE
        };

        // Set the stack pointer for double fault interrupts.
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            stack_start + STACK_SIZE
        };

        // Set the stack pointer for page fault interrupts.
        tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            VirtAddr::from_ptr(unsafe { &STACK }) + STACK_SIZE
        };

        // Set the stack pointer for general protection fault interrupts.
        tss.interrupt_stack_table[GENERAL_PROTECTION_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            VirtAddr::from_ptr(unsafe { &STACK }) + STACK_SIZE
        };

        tss
    };

    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        // Add entries to the Global Descriptor Table (GDT).
        let code = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss = gdt.add_entry(Descriptor::tss_segment(&TSS));
        let data = gdt.add_entry(Descriptor::kernel_data_segment());
        let user_code = gdt.add_entry(Descriptor::user_code_segment());
        let user_data = gdt.add_entry(Descriptor::user_data_segment());

        (
            gdt,
            Selectors {
                code,
                tss,
                data,
                user_code,
                user_data,
            },
        )
    };
}

#[allow(dead_code)]
struct Selectors {
    code: SegmentSelector,
    tss: SegmentSelector,
    data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub user_data: SegmentSelector,
}

/// Initializes the Global Descriptor Table (GDT) and Task State Segment (TSS).
pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS, DS, SS};
    use x86_64::instructions::tables::load_tss;

    // Load the GDT.
    GDT.0.load();

    unsafe {
        // Set the code segment register (CS) to the kernel code segment selector.
        CS::set_reg(GDT.1.code);

        // Set the data segment register (DS) to the kernel data segment selector.
        DS::set_reg(GDT.1.data);

        // Set the stack segment register (SS) to a null selector.
        SS::set_reg(SegmentSelector(0));

        // Load the task state segment (TSS).
        load_tss(GDT.1.tss);
    }
}