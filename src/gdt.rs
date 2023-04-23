use x86_64::VirtAddr;
use x86_64::instructions::tables::load_tss;
use x86_64::instructions::segmentation::{ CS, Segment };
use x86_64::structures::tss::TaskStateSegment;
use x86_64::structures::gdt::{ SegmentSelector, GlobalDescriptorTable, Descriptor };
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0; // 0th Entry Is Double Fault Stack

pub fn init() {
    GDT.0.load();
    unsafe { // Possible To Load Invalid Sectors
        CS::set_reg(GDT.1.code_selector); // Reload Code Segment Registers
        load_tss(GDT.1.tss_selector); // Load The Task State Segment
    }


}


lazy_static! { //           TSS 
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {  // IST
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE]; 

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}