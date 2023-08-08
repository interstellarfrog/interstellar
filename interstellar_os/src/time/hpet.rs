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

use x86_64::PhysAddr;

use crate::acpi::ACPI_INFO;

struct HpetRegisters<'a> {
    #[allow(dead_code)]
    pub general_capabilities_and_id: volatile::VolatilePtr<'a, u64>,
    #[allow(dead_code)]
    pub general_configuration: volatile::VolatilePtr<'a, u64>,
    #[allow(dead_code)]
    pub general_interrupt_status: volatile::VolatilePtr<'a, u64>,
    #[allow(dead_code)]
    pub main_counter_value: volatile::VolatilePtr<'a, u64>,
}

pub struct HPET<'a> {
    #[allow(dead_code)]
    registers: HpetRegisters<'a>,
}

impl Default for HPET<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl HPET<'_> {
    /// This should only be called if the HPET info in the ACPI tables is present
    pub fn new() -> Self {
        let acpi_info = ACPI_INFO.get().unwrap().lock();

        let hpet_info = acpi_info
            .hpet_info
            .as_ref()
            .expect("HPET info not available");

        let phys_mem_offet = crate::other::info::BOOT_INFO
            .get()
            .unwrap()
            .lock()
            .physical_memory_offset;

        let hpet_base_address = hpet_info.base_address as u64 + phys_mem_offet;

        let general_capabilities_and_id = unsafe {
            volatile::VolatilePtr::new(
                core::ptr::NonNull::new(PhysAddr::new(hpet_base_address).as_u64() as *mut u64)
                    .unwrap(),
            )
        };
        let general_configuration = unsafe {
            volatile::VolatilePtr::new(
                core::ptr::NonNull::new(
                    PhysAddr::new(hpet_base_address + 0x010).as_u64() as *mut u64
                )
                .unwrap(),
            )
        };
        let general_interrupt_status = unsafe {
            volatile::VolatilePtr::new(
                core::ptr::NonNull::new(
                    PhysAddr::new(hpet_base_address + 0x020).as_u64() as *mut u64
                )
                .unwrap(),
            )
        };
        let main_counter_value = unsafe {
            volatile::VolatilePtr::new(
                core::ptr::NonNull::new(
                    PhysAddr::new(hpet_base_address + 0x0F0).as_u64() as *mut u64
                )
                .unwrap(),
            )
        };

        let registers = HpetRegisters {
            general_capabilities_and_id,
            general_configuration,
            general_interrupt_status,
            main_counter_value,
        };

        HPET { registers }
    }
}
