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

use core::panic;

use crate::{other::info::BOOT_INFO, other::log::LOGGER};
use acpi::{
    sdt::Signature, AcpiHandler, AcpiTables, AmlTable, HpetInfo, PciConfigRegions, PhysicalMapping,
};
use alloc::vec::Vec;
use aml::{AmlContext, AmlError};
use os_units::Bytes;
use x86_64::{
    instructions::port::{PortReadOnly, PortWriteOnly},
    PhysAddr, VirtAddr,
};

#[derive(Clone)]
pub struct AcpiHandlerImpl;

impl AcpiHandler for AcpiHandlerImpl {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let virtual_address =
            crate::memory::map_address(PhysAddr::new(physical_address as u64), size);

        PhysicalMapping::new(
            physical_address,
            core::ptr::NonNull::new(virtual_address.as_mut_ptr()).unwrap(),
            size,
            size,
            self.clone(),
        )
    }

    fn unmap_physical_region<T>(region: &PhysicalMapping<Self, T>) {
        use x86_64::structures::paging::Mapper;
        use x86_64::structures::paging::{Page, PageSize, Size4KiB};

        let start = VirtAddr::new(region.virtual_start().as_ptr() as u64);
        let object_size = Bytes::new(region.region_length());

        let start_frame_addr = start.align_down(Size4KiB::SIZE);
        let end_frame_addr = (start + object_size.as_usize()).align_down(Size4KiB::SIZE);

        let num_pages = Bytes::new((end_frame_addr - start_frame_addr) as usize + 1)
            .as_num_of_pages::<Size4KiB>();

        let mut binding1 = crate::memory::MAPPER.lock();
        let mapper = binding1.as_mut().unwrap();

        for i in 0..num_pages.as_usize() {
            let page =
                Page::<Size4KiB>::containing_address(start_frame_addr + Size4KiB::SIZE * i as u64);

            let (_frame, flusher) = mapper.unmap(page).unwrap();
            flusher.flush();
        }
    }
}

pub fn init(rsdp_address: PhysAddr) -> AcpiTables<AcpiHandlerImpl> {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Initializing ACPI", file!(), line!());

    LOGGER.get().unwrap().lock().info("Initializing ACPI");

    let acpi_tables =
        unsafe { AcpiTables::from_rsdp(AcpiHandlerImpl, rsdp_address.as_u64() as usize) };

    match acpi_tables {
        Ok(mut acpi_tables) => {
            let _hpet_info = HpetInfo::new(&acpi_tables).unwrap();

            for sdt in &acpi_tables.sdts {
                if sdt.0 == &Signature::MADT {}
            }

            let dsdt = acpi_tables.dsdt.take().unwrap();

            let _pci = PciConfigRegions::new(&acpi_tables);

            let mut aml_tables = alloc::vec![&dsdt];

            let ssdts = &acpi_tables.ssdts;

            for ssdt in ssdts {
                aml_tables.append(&mut alloc::vec![ssdt]);
            }

            let _platform_info = acpi::platform::PlatformInfo::new(&acpi_tables);

            let _context = parse_aml_tables(aml_tables);

            acpi_tables
        }
        Err(e) => {
            panic!("Error Parsing ACPI tables, error code: {:#?}", e);
        }
    }
}

struct OsAmlHandler;

impl aml::Handler for OsAmlHandler {
    fn read_u8(&self, address: usize) -> u8 {
        unsafe {
            *((address + BOOT_INFO.get().unwrap().lock().physical_memory_offset as usize)
                as *const u8)
        }
    }

    fn read_u16(&self, address: usize) -> u16 {
        unsafe {
            *((address + BOOT_INFO.get().unwrap().lock().physical_memory_offset as usize)
                as *const u16)
        }
    }

    fn read_u32(&self, address: usize) -> u32 {
        unsafe {
            *((address + BOOT_INFO.get().unwrap().lock().physical_memory_offset as usize)
                as *const u32)
        }
    }

    fn read_u64(&self, address: usize) -> u64 {
        unsafe {
            *((address + BOOT_INFO.get().unwrap().lock().physical_memory_offset as usize)
                as *const u64)
        }
    }

    fn write_u8(&mut self, address: usize, value: u8) {
        let mut addr = unsafe {
            *((address + BOOT_INFO.get().unwrap().lock().physical_memory_offset as usize)
                as *const u8)
        };

        let volatile_addr = unsafe { volatile::VolatilePtr::new((&mut addr).into()) };
        volatile_addr.write(value);
    }

    fn write_u16(&mut self, address: usize, value: u16) {
        let mut addr = unsafe {
            *((address + BOOT_INFO.get().unwrap().lock().physical_memory_offset as usize)
                as *const u16)
        };
        let volatile_addr = unsafe { volatile::VolatilePtr::new((&mut addr).into()) };
        volatile_addr.write(value);
    }

    fn write_u32(&mut self, address: usize, value: u32) {
        let mut addr = unsafe {
            *((address + BOOT_INFO.get().unwrap().lock().physical_memory_offset as usize)
                as *const u32)
        };
        let volatile_addr = unsafe { volatile::VolatilePtr::new((&mut addr).into()) };
        volatile_addr.write(value);
    }

    fn write_u64(&mut self, address: usize, value: u64) {
        let mut addr = unsafe {
            *((address + BOOT_INFO.get().unwrap().lock().physical_memory_offset as usize)
                as *const u64)
        };
        let volatile_addr = unsafe { volatile::VolatilePtr::new((&mut addr).into()) };
        volatile_addr.write(value);
    }

    fn read_io_u8(&self, port: u16) -> u8 {
        let mut port = PortReadOnly::new(port);
        unsafe { port.read() }
    }

    fn read_io_u16(&self, port: u16) -> u16 {
        let mut port = PortReadOnly::new(port);
        unsafe { port.read() }
    }

    fn read_io_u32(&self, port: u16) -> u32 {
        let mut port = PortReadOnly::new(port);
        unsafe { port.read() }
    }

    fn write_io_u8(&self, port: u16, value: u8) {
        let mut port = PortWriteOnly::new(port);
        unsafe { port.write(value) }
    }

    fn write_io_u16(&self, port: u16, value: u16) {
        let mut port = PortWriteOnly::new(port);
        unsafe { port.write(value) }
    }

    fn write_io_u32(&self, port: u16, value: u32) {
        let mut port = PortWriteOnly::new(port);
        unsafe { port.write(value) }
    }

    fn read_pci_u8(&self, _segment: u16, _bus: u8, _device: u8, _function: u8, _offset: u16) -> u8 {
        todo!()
    }

    fn read_pci_u16(
        &self,
        _segment: u16,
        _bus: u8,
        _device: u8,
        _function: u8,
        _offset: u16,
    ) -> u16 {
        todo!()
    }

    fn read_pci_u32(
        &self,
        _segment: u16,
        _bus: u8,
        _device: u8,
        _function: u8,
        _offset: u16,
    ) -> u32 {
        todo!()
    }

    fn write_pci_u8(
        &self,
        _segment: u16,
        _bus: u8,
        _device: u8,
        _function: u8,
        _offset: u16,
        _value: u8,
    ) {
        todo!()
    }

    fn write_pci_u16(
        &self,
        _segment: u16,
        _bus: u8,
        _device: u8,
        _function: u8,
        _offset: u16,
        _value: u16,
    ) {
        todo!()
    }

    fn write_pci_u32(
        &self,
        _segment: u16,
        _bus: u8,
        _device: u8,
        _function: u8,
        _offset: u16,
        _value: u32,
    ) {
        todo!()
    }
}

/// Parse the given AML tables
///
/// # Warning
///
/// Only call this function one time with all AML tables
fn parse_aml_tables(aml_tables: Vec<&AmlTable>) -> Result<AmlContext, AmlError> {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("creating AML context", file!(), line!());
    let mut context = aml::AmlContext::new(
        alloc::boxed::Box::new(OsAmlHandler {}),
        aml::DebugVerbosity::All,
    );
    for aml_table in aml_tables {
        LOGGER.get().unwrap().lock().trace(
            "Making AML bytecode stream from raw pointer",
            file!(),
            line!(),
        );
        let aml_bytecode: &[u8] = unsafe {
            core::slice::from_raw_parts(
                (aml_table.address
                    + BOOT_INFO.get().unwrap().lock().physical_memory_offset as usize)
                    as *const u8,
                aml_table.length as usize,
            )
        };

        LOGGER
            .get()
            .unwrap()
            .lock()
            .trace("Parsing AML table", file!(), line!());
        context
            .parse_table(aml_bytecode)
            .expect("Could not parse AML table");
    }
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace("Succesfully parsed AML tables", file!(), line!());
    Ok(context)
}
