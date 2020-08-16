#[allow(clippy::inline_always)]
#[inline(always)] // Just an asm instruction, should always be inlined
pub fn wfe() {
    unsafe {
        asm!("wfe", options(nomem, nostack));
    }
}

#[allow(clippy::inline_always)]
#[inline(always)] // Just an asm instruction, should always be inlined
pub fn sev() {
    unsafe {
        asm!("sev", options(nomem, nostack));
    }
}

#[allow(clippy::inline_always)]
#[inline(always)] // Just an asm instruction, should always be inlined
pub fn wfi() {
    unsafe {
        asm!("wfi", options(nomem, nostack));
    }
}

#[allow(clippy::inline_always)]
#[inline(always)] // Just an asm instruction, should always be inlined
pub fn isb() {
    unsafe {
        asm!("isb");
    }
}

#[allow(clippy::inline_always)]
#[inline(always)] // Just an asm instruction, should always be inlined
pub fn eret() -> ! {
    unsafe { asm!("eret", options(noreturn)) }
}

/// Set the stack pointer.
#[allow(clippy::inline_always)]
#[inline(always)] // Just an asm instruction, should always be inlined
pub fn set_sp(value: usize) {
    unsafe {
        asm!("mov sp, {}", in(reg) value);
    }
}

pub mod lr {
    #[allow(clippy::inline_always)]
    #[inline(always)] // Just an asm instruction, should always be inlined
    pub fn read() -> usize {
        let value;
        unsafe {
            asm!("mov {}, lr", out(reg) value);
        }
        value
    }
    #[allow(clippy::inline_always)]
    #[inline(always)] // Just an asm instruction, should always be inlined
    pub fn write(value: usize) {
        unsafe {
            asm!("mov lr, {}", in(reg) value);
        }
    }
}

/// Hangs the CPU by looping a `wfe` (Wait for Event) instruction.
#[allow(clippy::inline_always)]
#[inline(always)] // Just an asm instruction, should always be inlined
pub fn hang_cpu() -> ! {
    loop {
        wfe();
    }
}
