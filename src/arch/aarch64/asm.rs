#[allow(clippy::inline_always)]
#[inline(always)]
pub fn wfe() {
    unsafe {
        asm!("wfe", options(nomem, nostack));
    }
}

#[allow(clippy::inline_always)]
#[inline(always)]
pub fn sev() {
    unsafe {
        asm!("sev", options(nomem, nostack));
    }
}

#[allow(clippy::inline_always)]
#[inline(always)]
pub fn wfi() {
    unsafe {
        asm!("wfi", options(nomem, nostack));
    }
}

#[allow(clippy::inline_always)]
#[inline(always)]
pub fn isb() {
    unsafe {
        asm!("isb");
    }
}

#[allow(clippy::inline_always)]
#[inline(always)]
pub fn eret() -> ! {
    unsafe { asm!("eret", options(noreturn)) }
}

#[allow(clippy::inline_always)]
#[inline(always)]
pub fn set_sp(value: usize) {
    unsafe {
        asm!("mov sp, {}", in(reg) value);
    }
}

pub mod lr {
    #[allow(clippy::inline_always)]
    #[inline(always)]
    pub fn read() -> usize {
        let value;
        unsafe {
            asm!("mov {}, lr", out(reg) value);
        }
        value
    }
    #[allow(clippy::inline_always)]
    #[inline(always)]
    pub fn write(value: usize) {
        unsafe {
            asm!("mov lr, {}", in(reg) value);
        }
    }
}

#[allow(clippy::inline_always)]
#[inline(always)]
pub fn hang_cpu() -> ! {
    loop {
        wfe();
    }
}
