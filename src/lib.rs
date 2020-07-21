#![no_std]
#![cfg_attr(test, no_main)]
#![feature(
    custom_test_frameworks,
    abi_x86_interrupt,
    toowned_clone_into,
    alloc_error_handler,
    const_fn,
    const_in_array_repeat_expressions,
    wake_trait,
    trait_alias,
    shrink_to
)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![deny(clippy::all, clippy::pedantic)]
#![allow(clippy::new_without_default, clippy::must_use_candidate)]

extern crate alloc;

pub mod allocator;
pub mod arch;
pub mod task;

use log::{Log, Metadata, Record};

static LOGGER: Logger = Logger;

pub struct Logger;

impl Logger {
    pub fn init(level: log::LevelFilter) {
        log::set_logger(&LOGGER).expect("Could not init logger");
        log::set_max_level(level);
    }
}

impl Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            #[cfg(feature = "log_vga")]
            {
                use log::Level;
                use vga::colors::{Color16, TextModeColor};

                let color = match record.level() {
                    Level::Error => TextModeColor::new(Color16::Black, Color16::Red),
                    Level::Warn => TextModeColor::new(Color16::Yellow, Color16::Black),
                    Level::Info => TextModeColor::new(Color16::LightBlue, Color16::Black),
                    _ => TextModeColor::new(Color16::White, Color16::Black),
                };
                crate::println_colored!(color, "[{:5}] {}", record.level(), record.args());
            }
            #[cfg(feature = "log_serial")]
            {
                crate::serial_println!("[{:5}] {}", record.level(), record.args());
            }
        }
    }

    fn flush(&self) {}
}

// Test related things

#[cfg(test)]
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;

pub fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }
    exit_qemu(QemuExitCode::Success);
}

#[allow(unused_variables)]
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

#[cfg(test)]
entry_point!(test_kernel_main);

/// Entry point for `cargo xtest`
#[cfg(test)]
fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
    arch::x86_64::start(boot_info);
    test_main();
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
