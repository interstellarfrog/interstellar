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

use crate::other::log::LOGGER;
use core::arch::asm;

/// Checks if interrupts are enabled.
///
/// Returns `true` if interrupts are enabled, `false` otherwise.
#[inline]
pub fn interrupts_enabled() -> bool {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Cheking if interrupts are enabled"), file!(), line!());
    let r: u64;

    // Push Flags Reg Value Onto Stack Pop Into r
    unsafe {
        asm!("pushfq; pop {}", out(reg) r, options(nomem, preserves_flags));
    }

    if r & (1 << 9) != 0 {
        // If bit 9 From Right != 0
        true // Interrupts Enabled
    } else {
        false // Interrupts Disabled
    }
}

/// Enables interrupts.
#[inline]
pub fn interrupts_enable() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Enabling interrupts"), file!(), line!());
    unsafe {
        asm!("sti", options(nomem, nostack));
    }
}

/// Enables interrupts and halts the CPU.
#[inline]
pub fn interrupts_enable_and_hlt() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Enabling interrupts and halting"), file!(), line!());
    unsafe {
        asm!("sti; hlt", options(nomem, nostack));
    }
}

/// Disables interrupts.
#[inline]
pub fn interrupts_disable() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Disabling interrupts"), file!(), line!());
    unsafe {
        asm!("cli", options(nomem, nostack));
    }
}

/// Executes a closure with interrupts disabled, and restores the interrupt state afterwards.
///
/// # Arguments
///
/// * `f` - The closure to be executed.
///
/// # Returns
///
/// The return value of the closure `f`.
pub fn interrupts_without<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    LOGGER.get().unwrap().lock().trace(
        Some("Executing a function without interrupts"),
        file!(),
        line!(),
    );
    if interrupts_enabled() {
        interrupts_disable();
        let ret = f();
        interrupts_enable();
        ret
    } else {
        f()
    }
}

/// Halts the CPU.
pub fn hlt() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Halting CPU"), file!(), line!());
    unsafe {
        asm!("hlt", options(nomem, nostack, preserves_flags));
    }
}

/// Loops until the next interrupt occurs, saving CPU usage.
///
/// This function does not return.
#[inline]
pub fn hlt_loop() -> ! {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Entering halt loop"), file!(), line!());
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

/// Raises a breakpoint exception.
#[inline]
pub fn exception_breakpoint() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Causing breakpoint exception"), file!(), line!());
    unsafe {
        asm!("int3", options(nomem, nostack));
    }
}

/// No operation (does nothing).
#[inline]
pub fn nop() {
    LOGGER
        .get()
        .unwrap()
        .lock()
        .trace(Some("Executing nop instruction"), file!(), line!());
    unsafe {
        asm!("nop", options(nomem, nostack, preserves_flags));
    }
}
