#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(hakkero::test::runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use hakkero::{serial_print, serial_println};

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    hakkero::test::panic_handler(info)
}

#[test_case]
fn test_boot() {
    serial_print!("test_boot... ");
    serial_println!("[ok]");
}
