#![no_std]
#![no_main]
#![feature(custom_test_frameworks, asm, alloc_prelude)]
#![test_runner(hakkero::test::runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
extern crate alloc;

use core::panic::PanicInfo;
use hakkero::arch::x86_64::{
    device::vga::Readline,
    start,
    task::{handle_scancodes, DecodedKeyStream},
};
use hakkero::task::{spawn_task, Executor, Task};

entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Initialize phase
    start(boot_info);

    // We run tests before everything to avoid interference
    #[cfg(test)]
    test_main();

    some_info();

    let mut executor = Executor::new();
    executor.spawn(Task::new(start_handlers()));
    log::info!("Welcome to Hakkero OS!\n");
    executor.run();
}

fn some_info() {
    use hakkero::allocator::*;
    use log::info;

    info!("Heap start: {}", HEAP_START);
    info!("Heap size: {}", HEAP_SIZE);
    info!("Heap usage: {}", ALLOCATOR.lock().used_heap());
}

async fn start_handlers() {
    spawn_task(Task::new(handle_scancodes()));
    spawn_task(Task::new(handle_decoded_keys()));
}

async fn handle_decoded_keys() {
    use futures_util::stream::StreamExt;

    let mut queue = DecodedKeyStream;
    let mut rl = Readline::new();

    while let Some(key) = queue.next().await {
        if let Some(s) = rl.handle_key(key) {
            hakkero::println!("{}", s);
        }
    }
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    hakkero::arch::x86_64::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    hakkero::test::panic_handler(info)
}
