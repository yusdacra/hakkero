//! `AArch64` specific code.
pub mod asm;
pub mod board;
pub mod device;
pub mod register;

pub use asm::hang_cpu;

use core::ops::Range;

/// Initializes the required peripherals.
///
/// # Safety
/// Must only be called once.
#[allow(clippy::inline_always)]
#[inline(always)] // Inline because it will be only used once anyways
pub unsafe fn init() {
    // Have to include this so logging works lol
    crate::serial_print!("");
    crate::logger::init();
    // Also this too
    log::info!("Initialized all peripherals!");
}

#[doc(hidden)]
pub(crate) macro __entry_point($path:path) {
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

            let entry: fn() -> ! = $path;
            entry()
        }

        // If we aren't core 0 then hang
        asm::hang_cpu()
    }
}

/// Make an entry point. This macro checks the signature of the provided
/// function to make sure it's correct.
pub macro entry_point($path:path) {
    $crate::arch::__entry_point!(kernel_init);

    fn kernel_init() -> ! {
        let entry: fn() -> ! = $path;
        unsafe { $crate::arch::init() }
        entry()
    }
}

/// Return the bss section symbol addresses as a `Range`.
///
/// # Safety
/// - The symbol-provided addresses must be valid.
/// - The symbol-provided addresses must be usize aligned.
#[allow(clippy::inline_always)]
#[inline(always)] // Inline because it will be only used once anyways
pub unsafe fn bss_range() -> Range<*mut usize> {
    ld_symbol_range!(__bss_start, __bss_end)
}

/// Return the stack section symbol addresses as a `Range`.
///
/// # Safety
/// - The symbol-provided addresses must be valid.
/// - The symbol-provided addresses must be usize aligned.
#[allow(clippy::inline_always)]
#[inline(always)] // Inline because it will be only used once anyways
pub unsafe fn stack_range() -> Range<*mut usize> {
    ld_symbol_range!(__stack_start, __stack_end)
}

/// Return the specified "$start" and "$end" symbols from the linker
/// as a `Range`.
macro ld_symbol_range($start:ident, $end:ident) {{
    extern "C" {
        static mut $start: usize;
        static mut $end: usize;
    }

    Range {
        start: &mut $start,
        end: &mut $end,
    }
}}
