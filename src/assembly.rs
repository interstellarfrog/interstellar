use core::arch::asm;



#[inline]
pub fn interrupts_enabled() -> bool {
    let r: u64;

    // Push Flags Reg Value Onto Stack Pop Into r
    unsafe{ asm!("pushfq; pop {}", out(reg) r, options(nomem, preserves_flags)); }
    
    if r & (1 << 9) != 0 { // If bit 9 From Right != 0
        return true; // Interrupts Enabled
    } else {
        return false; // Interrupts Disabled
    }
}

#[inline]
pub fn interrupts_enable() {
    unsafe { asm!("sti", options(nomem, nostack)); }
}

#[inline]
pub fn interrupts_enable_and_hlt() {
    unsafe{ asm!("sti; hlt", options(nomem, nostack)); }
}



#[inline]
pub fn interrupts_disable() {
    unsafe{ asm!("cli", options(nomem, nostack)); }
}


#[inline]
pub fn interrupts_without<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    if interrupts_enabled() {
        interrupts_disable();
        let ret = f();
        interrupts_enable();
        ret
    } else {
        let ret = f();
        ret
    }
}

pub fn hlt() {
    unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)); }
}

#[inline]
pub fn hlt_loop() -> ! { // Loop Until Next Interrupt - Saves CPU Percentage
    loop {
        unsafe { asm!("hlt", options(nomem, nostack, preserves_flags)); }
    }
}

#[inline]
pub fn exception_breakpoint() {
    unsafe{ asm!("int3", options(nomem, nostack)); }
}


#[inline]
pub fn nop() {
    unsafe{ asm!("nop", options(nomem, nostack, preserves_flags)); }
}