#![no_std]
#![no_main]
#![feature(custom_test_frameworks, llvm_asm)]
#![test_runner(hakkero::test::runner)]
#![reexport_test_harness_main = "test_main"]

// Only compile on systems where we have heap setup
#[cfg(target_arch = "x86_64")]
extern crate alloc;

use core::panic::PanicInfo;
#[cfg(target_arch = "aarch64")]
use hakkero::arch::aarch64::start;
#[cfg(target_arch = "x86_64")]
use {
    bootloader::{entry_point, BootInfo},
    hakkero::{
        arch::x86_64::{
            device::vga::Readline,
            start,
            task::{handle_scancodes, DecodedKeyStream},
        },
        task::{spawn_task, Executor, Task},
    },
};

#[cfg(target_arch = "x86_64")]
entry_point!(kernel_main);
#[cfg(target_arch = "x86_64")]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Initialize phase
    start(boot_info);

    // We run tests before everything to avoid interference
    #[cfg(test)]
    test_main();

    heap_info();

    let mut executor = Executor::new();
    executor.spawn(Task::new(start_handlers()));
    log::info!("Welcome to Hakkero OS!\n");
    executor.run();
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
extern "C" fn kernel_main() -> ! {
    start();

    hakkero::console_println!("Hello world!");

    panic!()
}

// Only compile on systems where we have heap setup
#[cfg(target_arch = "x86_64")]
fn heap_info() {
    use hakkero::allocator::*;
    use log::info;

    info!("Heap start: {}", HEAP_START);
    info!("Heap size: {}", HEAP_SIZE);
    info!("Heap usage: {}", ALLOCATOR.lock().used_heap());
}

#[cfg(target_arch = "x86_64")]
async fn start_handlers() {
    spawn_task(Task::new(handle_scancodes()));
    spawn_task(Task::new(async {
        use futures_util::stream::StreamExt;

        let mut queue = DecodedKeyStream;
        let mut rl = Readline::new();

        while let Some(key) = queue.next().await {
            if let Some(s) = rl.handle_key(key) {
                hakkero::println!("{}", s);
            }
        }
    }));
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    #[cfg(target_arch = "x86_64")]
    {
        hakkero::arch::x86_64::hlt_loop()
    }
    #[cfg(target_arch = "aarch64")]
    {
        hakkero::arch::aarch64::wait_forever()
    }
}

// Only test on x86_64
#[cfg(target_arch = "x86_64")]
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    hakkero::test::panic_handler(info)
}
