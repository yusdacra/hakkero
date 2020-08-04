//! `AArch64` specific code.
pub mod asm;
pub mod board;
pub mod device;
pub mod register;

pub use asm::hang_cpu;

use core::ops::Range;

/// Make an entry point. This macro checks the signature of the provided
/// function to make sure it's correct.
#[macro_export]
macro_rules! entry_point {
    ($path:path) => {
        #[no_mangle]
        #[naked]
        pub extern "C" fn _start() -> ! {
            use $crate::arch::{asm, register};

            if register::core_id() == 0 {
                use $crate::arch::{asm, bss_range, stack_range};

                unsafe {
                    // zero bss
                    $crate::memory::zero_volatile(bss_range());

                    // setup stack
                    asm::set_sp(stack_range().start as usize);
                }

                // TODO: fix logging
                // $crate::misc::init();

                let entry: fn() -> ! = $path;
                entry()
            }

            // If we aren't core 0 then hang
            asm::hang_cpu()
        }
    };
}

/// # Safety
/// - The symbol-provided addresses must be valid.
/// - The symbol-provided addresses must be usize aligned.
#[allow(clippy::inline_always)]
#[inline(always)] // Inline because it will be only used once anyways
pub unsafe fn bss_range() -> Range<*mut usize> {
    ld_symbol_range!(usize, __bss_start, __bss_end)
}

/// # Safety
/// - The symbol-provided addresses must be valid.
/// - The symbol-provided addresses must be usize aligned.
#[allow(clippy::inline_always)]
#[inline(always)] // Inline because it will be only used once anyways
pub unsafe fn stack_range() -> Range<*mut usize> {
    ld_symbol_range!(usize, __stack_start, __stack_end)
}

macro ld_symbol_range($size:ty, $start:ident, $end:ident) {{
    extern "C" {
        static mut $start: usize;
        static mut $end: usize;
    }

    Range {
        start: &mut $start,
        end: &mut $end,
    }
}}
