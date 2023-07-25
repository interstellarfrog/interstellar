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

/// The Code In This File Is For Handling CPU Exceptions And Interrupts - 0 Division errors etc. And legacy Keyboard Input etc.
use crate::{gdt, other::log::LOGGER};
use acpi::PlatformInfo;
use handlers::*;
use lazy_static::lazy_static;
use x86_64::structures::idt::InterruptDescriptorTable;

use x86_64::PhysAddr;

use spinning_top::Spinlock;

use conquer_once::spin::OnceCell;

use x2apic::{
    ioapic::{IoApic, IrqFlags, RedirectionTableEntry},
    lapic::{LocalApic, LocalApicBuilder, TimerDivide},
};

use acpi::{platform::interrupt::Apic as ApicInfo, InterruptModel};

mod handlers;

pub static LAPIC_BASE: OnceCell<u64> = OnceCell::uninit();

pub static LAPIC: OnceCell<Spinlock<LocalApic>> = OnceCell::uninit();

pub static IOAPIC: OnceCell<Spinlock<IoApic>> = OnceCell::uninit();

/*
Vector |Exception/Interrupt |Mnemonic |Cause
0 |Divide-by-Zero-Error |#DE |DIV, IDIV, AAM instructions
1 |Debug |#DB |Instruction accesses and data accesses
2 |Non-Maskable-Interrupt |#NMI |External NMI signal
3 |Breakpoint |#BP |INT3 instruction
4 |Overflow |#OF |INTO instruction
5 |Bound-Range |#BR |BOUND instruction
6 |Invalid-Opcode |#UD |Invalid instructions
7 |Device-Not-Available |#NM |x87 instructions
8 |Double-Fault |#DF |Exception during the handling of another exception or interrupt
9 |Coprocessor-Segment-Overrun |— |Unsupported (Reserved)
10 |Invalid-TSS |#TS |Task-state segment access and task switch
11 |Segment-Not-Present |#NP |Segment register loads
12 |Stack |#SS |SS register loads and stack references
13 |General-Protection |#GP |Memory accesses and protection checks
14 |Page-Fault |#PF |Memory accesses when paging enabled
15 |Reserved |—
16 |x87 Floating-Point Exception-Pending |#MF |x87 floating-point instructions
17 |Alignment-Check |#AC |Misaligned memory accesses
18 |Machine-Check |#MC |Model specific
19 |SIMD Floating-Point |#XF |SSE floating-point instructions
20 |Reserved |—
21 |Control-Protection Exception |#CP |RET/IRET or other control transfer
22-27 |Reserved |—
28 |Hypervisor Injection Exception |#HV |Event injection
29 |VMM Communication Exception |#VC |Virtualization event
30 |Security Exception |#SX |Security-sensitive event in host
31 |Reserved |—
0–255 |External Interrupts (Maskable) |#INTR |External interrupts
0–255 |Software Interrupts |— |INTn instruction
*/

// Lazy-static IDT (Interrupt Descriptor Table) for handling interrupts
lazy_static! {
    /// The kernels interrupt descriptor table
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new(); // Create a new IDT
        //################################################
        //#                Exceptions
        //################################################
        // 0-31 = 32 total

        idt.divide_error.set_handler_fn(divide_by_zero_fault_handler);
        idt.debug.set_handler_fn(debug_handler);
        idt.non_maskable_interrupt.set_handler_fn(non_masked_interrupt_handler);
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.overflow.set_handler_fn(overflow_handler);
        idt.bound_range_exceeded.set_handler_fn(bound_range_exceeded_handler);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_fault_handler);
        idt.device_not_available.set_handler_fn(device_not_available_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        // Co-Processor-Segment-Overrun
        idt.invalid_tss.set_handler_fn(invalid_tss_fault_handler);
        idt.segment_not_present.set_handler_fn(segment_not_present_handler);
        idt.stack_segment_fault.set_handler_fn(stack_segment_fault_handler);
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        // Reserved
        idt.x87_floating_point.set_handler_fn(x87_floating_point_handler);
        idt.alignment_check.set_handler_fn(alignment_check_handler);
        idt.machine_check.set_handler_fn(machine_check_handler);
        idt.simd_floating_point.set_handler_fn(simd_floating_point_handler);
        // Reserved
        // Control-Protection Exception
        // Reserved
        // Hypervisor Injection Exception
        idt.vmm_communication_exception.set_handler_fn(vmm_communication_handler);
        idt.security_exception.set_handler_fn(security_exception_fault_handler); // Set the handler for the security exception
        // Reserved

        unsafe {
            // Set the stack index for the double fault handler to switch the stack
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        //################################################
        //#                APIC Interrupts
        //################################################

        idt[IoApicInterruptIndex::Pit.as_usize()].set_handler_fn(pit_interrupt_handler);
        idt[IoApicInterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt[IoApicInterruptIndex::Mouse.as_usize()].set_handler_fn(mouse_interrupt_handler);

        idt[LApicInterruptIndex::ApicError.as_usize()].set_handler_fn(error_interrupt_handler);
        idt[LApicInterruptIndex::Timer.as_usize()].set_handler_fn(apic_timer_interrupt_handler);
        idt[LApicInterruptIndex::Spurious.as_usize()].set_handler_fn(spurious_interrupt_handler);



        idt
    };
}

/// Initialize interrupts and exceptions
pub fn init(acpi_platform_info: &PlatformInfo) {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Initializing interrupts", file!(), line!());

    LOGGER.get().unwrap().lock().info("Initializing interrupts");

    {
        // Disable PS/2 to not mess up initialization
        let mut cmd = x86_64::instructions::port::Port::<u8>::new(0x64);
        unsafe {
            cmd.write(0xad); // disable keyboard port
            cmd.write(0xa7); // disable mouse port
        }
        // flush PS/2 buffer
        let mut data = x86_64::instructions::port::Port::<u8>::new(0x60);
        unsafe {
            // if bit 0 is unset, the buffer is empty
            while (data.read() & 0x1) == 1 {}
        }
    }
    x86_64::instructions::interrupts::disable();

    IDT.load();

    if let InterruptModel::Apic(ref apic_info) = acpi_platform_info.interrupt_model {
        init_lapic(apic_info);

        init_ioapic(apic_info);

        // Enable PIT  - FIX ME!!!!!

        //let mut command_port = x86_64::instructions::port::Port::new(0x43);

        //unsafe {
        //    command_port.write(0b00110110u8); // set operating mode 2
        //}

        //let mut data_port = x86_64::instructions::port::Port::new(0x40);

        //unsafe {
        //    data_port.write((1193 & 0xFF) as u8); // lower byte of reload value
        //    data_port.write((1193 >> 8) as u8); // upper byte of reload value
        //}

        //unsafe { LAPIC.get().unwrap().lock().disable_timer() };

        //unsafe { LAPIC.get().unwrap().lock().set_timer_initial(1) };

        //unsafe { LAPIC.get().unwrap().lock().enable_timer() };

        // Wait for 10 ms using PIT

        // disable timer

        // check how many ticks the timer executed

        // set new timer settings

        // enable timer
    } else {
        LOGGER
            .get()
            .unwrap()
            .lock()
            .trace("Interrupt model not supported", file!(), line!());
        panic!("interstellar OS relys on APIC which your processor does not support")
    };

    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Enabling interrupts", file!(), line!());

    LOGGER.get().unwrap().lock().info("Enabling interrupts");

    x86_64::instructions::interrupts::enable(); // Enable Interrupts
}

fn init_lapic(apic_info: &ApicInfo) {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Initializing LAPIC", file!(), line!());

    LOGGER.get().unwrap().lock().info("Initializing LAPIC");
    unsafe {
        // Disable PIC so it doesn't interfere with LAPIC/IOPICs
        PICS.lock().disable();
    }

    LAPIC_BASE.init_once(|| apic_info.local_apic_address);
    LAPIC.init_once(|| {
        let apic_virtual_address =
            crate::memory::map_address(PhysAddr::new(apic_info.local_apic_address), 4096);

        let mut lapic = LocalApicBuilder::new()
            .set_xapic_base(apic_virtual_address.as_u64())
            .spurious_vector(LApicInterruptIndex::Spurious.as_usize())
            .error_vector(LApicInterruptIndex::ApicError.as_usize())
            .timer_divide(TimerDivide::Div16)
            .timer_vector(LApicInterruptIndex::Timer.as_usize())
            .timer_initial(10_000_000)
            .build()
            .unwrap_or_else(|e| panic!("{}", e));

        unsafe {
            lapic.enable();
        }

        unsafe {
            LOGGER.get().unwrap().lock().info(&alloc::format!(
                "apic id: {}, version: {}",
                lapic.id(),
                lapic.version()
            ));
        }

        Spinlock::new(lapic)
    });
}

fn init_ioapic(apic_info: &ApicInfo) {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Initializing IOPIC", file!(), line!());

    LOGGER.get().unwrap().lock().info("Initializing IOPIC");

    IOAPIC.init_once(|| {
        let lapic = LAPIC
            .get()
            .expect("should have the LAPIC initialized")
            .lock();

        let io_apic_virtual_address =
            crate::memory::map_address(PhysAddr::new(apic_info.io_apics[0].address as u64), 4096);

        let mut ioapic = unsafe { IoApic::new(io_apic_virtual_address.as_u64()) };

        unsafe {
            ioapic.init(IOAPIC_INTERRUPT_INDEX_OFFSET);

            LOGGER.get().unwrap().lock().info(&alloc::format!(
                "ioapic id: {}, version: {}",
                ioapic.id(),
                ioapic.version()
            ));

            register_io_apic_entry(
                &mut ioapic,
                lapic.id() as u8,
                IoApicInterruptIndex::Pit.as_u8(),
                IoApicTableIndex::Pit.into(),
            );

            register_io_apic_entry(
                &mut ioapic,
                lapic.id() as u8,
                IoApicInterruptIndex::Keyboard.as_u8(),
                IoApicTableIndex::Keyboard.into(),
            );
            register_io_apic_entry(
                &mut ioapic,
                lapic.id() as u8,
                IoApicInterruptIndex::Mouse.as_u8(),
                IoApicTableIndex::Mouse.into(),
            );
        }

        drop(lapic);

        Spinlock::new(ioapic)
    });

    LOGGER.get().unwrap().lock().info("IOAPIC initialized");
}

fn register_io_apic_entry(ioapic: &mut IoApic, lapic_id: u8, int_index: u8, irq_index: u8) {
    let mut entry = RedirectionTableEntry::default();
    entry.set_mode(x2apic::ioapic::IrqMode::Fixed);
    entry.set_dest(lapic_id);
    entry.set_vector(int_index);
    entry.set_flags(IrqFlags::LEVEL_TRIGGERED | IrqFlags::LOW_ACTIVE | IrqFlags::MASKED);
    unsafe {
        ioapic.set_table_entry(irq_index, entry);
        ioapic.enable_irq(irq_index);
    }
}
