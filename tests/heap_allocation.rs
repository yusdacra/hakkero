#![cfg(target_arch = "x86_64")]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(test::runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use hakkero::{arch::entry_point, serial_print, serial_println, test};

entry_point!(main);

fn main() -> ! {
    test_main();
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test::panic_handler(info)
}

/// Tests
use alloc::boxed::Box;

#[test_case]
fn simple_allocation() {
    serial_print!("simple_allocation... ");
    let heap_value = Box::new(41);
    assert_eq!(*heap_value, 41);
    serial_println!("[ok]");
}

use alloc::vec::Vec;

#[test_case]
fn large_vec() {
    serial_print!("large_vec... ");
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
    serial_println!("[ok]");
}

#[test_case]
fn many_boxes() {
    serial_print!("many_boxes... ");
    for i in 0..10_000 {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    serial_println!("[ok]");
}

#[test_case]
fn many_boxes_long_lived() {
    serial_print!("many_boxes_long_lived... ");
    let long_lived = Box::new(1); // new
    for i in 0..hakkero::allocator::HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
    assert_eq!(*long_lived, 1); // new
    serial_println!("[ok]");
}
