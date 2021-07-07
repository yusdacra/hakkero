//! Panic handlers are included here.

use core::panic::PanicInfo;

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    hakkero::arch::hang_cpu();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    hakkero::test::panic_handler(info)
}
