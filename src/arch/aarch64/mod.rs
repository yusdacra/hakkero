pub mod asm;
pub mod device;
pub mod memory;

use core::ops::Range;

#[allow(clippy::inline_always)]
#[inline(always)] // Inline because it will be only used once anyways
pub fn start() {
    unsafe {
        memory::zero_volatile(bss_range());
    }
    crate::common::init();
}

pub fn wait_forever() -> ! {
    loop {
        asm::wfi();
    }
}

// Jump to kernel stuff

global_asm!(include_str!("start.S"));

/// # Safety
/// - The symbol-provided addresses must be valid.
/// - The symbol-provided addresses must be usize aligned.
#[allow(clippy::inline_always)]
#[inline(always)] // Inline because it will be only used once anyways
unsafe fn bss_range() -> Range<*mut usize> {
    extern "C" {
        // Boundaries of the .bss section, provided by linker script symbols.
        static mut __bss_start: usize;
        static mut __bss_end: usize;
    }

    Range {
        start: &mut __bss_start,
        end: &mut __bss_end,
    }
}
