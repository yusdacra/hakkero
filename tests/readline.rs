#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(hakkero::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use hakkero::readline::Readline;
use hakkero::{serial_print, serial_println};

entry_point!(main);

fn main(boot_info: &'static BootInfo) -> ! {
    hakkero::init();
    hakkero::init_heap(boot_info);

    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    hakkero::test_panic_handler(info)
}

#[test_case]
fn test_handle_character() {
    serial_print!("handle_character...");
    let mut rl = Readline::new();
    rl.handle_character('a');
    assert_eq!(rl.retrieve_data().unwrap(), "a");
    serial_println!("[ok]");
}

#[test_case]
fn test_handle_newline() {
    serial_print!("handle_newline...");
    let mut rl = Readline::new();
    rl.handle_character('\n');
    assert_eq!(rl.retrieve_data(), None);
    serial_println!("[ok]");
}

#[test_case]
fn test_handle_backspace() {
    serial_print!("handle_backspace...");
    let mut rl = Readline::new();
    rl.handle_character('a');
    rl.handle_character('\u{8}');
    assert_eq!(rl.retrieve_data(), None);
    serial_println!("[ok]")
}

#[test_case]
fn test_handle_backspace_at_zero() {
    serial_print!("handle_backspace_at_zero...");
    let mut rl = Readline::new();
    rl.handle_character('\u{8}');
    assert_eq!(rl.retrieve_data(), None);
    serial_println!("[ok]")
}
