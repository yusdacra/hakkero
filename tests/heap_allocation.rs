#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(hakkero::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
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

/// Tests
use alloc::boxed::Box;

#[test_case]
fn simple_allocation(sp: &mut hakkero::serial::SerialPort) {
    serial_print!(sp, "simple_allocation... ");
    let heap_value = Box::new(41);
    assert_eq!(*heap_value, 41);
    serial_println!(sp, "[ok]");
}

use alloc::vec::Vec;

#[test_case]
fn large_vec(sp: &mut hakkero::serial::SerialPort) {
    serial_print!(sp, "large_vec... ");
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
    serial_println!(sp, "[ok]");
}

#[test_case]
fn many_boxes(sp: &mut hakkero::serial::SerialPort) {
    serial_print!(sp, "many_boxes... ");
    for i in 0..10_000 {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    serial_println!(sp, "[ok]");
}

#[test_case]
fn many_boxes_long_lived(sp: &mut hakkero::serial::SerialPort) {
    serial_print!(sp, "many_boxes_long_lived... ");
    let long_lived = Box::new(1); // new
    for i in 0..hakkero::allocator::HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1); // new
    serial_println!(sp, "[ok]");
}
