#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(hakkero::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use hakkero::{serial_print, serial_println};

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    hakkero::test_panic_handler(info)
}

#[test_case]
fn test_boot() {
    serial_print!("test_boot... ");
    serial_println!("[ok]");
}
