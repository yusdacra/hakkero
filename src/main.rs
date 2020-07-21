#![no_std]
#![no_main]
#![feature(custom_test_frameworks, asm, alloc_prelude)]
#![test_runner(hakkero::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
extern crate alloc;

use alloc::prelude::v1::*;
use core::panic::PanicInfo;
use hakkero::task::{
    executor::{spawn_task, Executor},
    keyboard, Task,
};

entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Initialize phase
    hakkero::arch::x86_64::start(boot_info);

    // We run tests before everything to avoid interference
    #[cfg(test)]
    test_main();

    some_info();
    tutorial_test_things();

    let mut executor = Executor::new();
    executor.spawn(Task::new(start_handlers()));
    log::info!("Welcome to Hakkero OS!\n");
    executor.run();
}

fn some_info() {
    use log::info;

    info!("Heap start: {}", hakkero::allocator::HEAP_START);
    info!("Heap size: {}", hakkero::allocator::HEAP_SIZE);
}

async fn start_handlers() {
    spawn_task(Task::new(keyboard::handle_scancodes()));
    spawn_task(Task::new(handle_decoded_keys()));
}

async fn handle_decoded_keys() {
    use futures_util::stream::StreamExt;

    let mut queue = hakkero::task::keyboard::DecodedKeyStream;
    let mut rl = hakkero::arch::x86_64::device::vga::Readline::default();

    while let Some(key) = queue.next().await {
        if let Some(s) = rl.handle_key(key) {
            hakkero::println!("{}", s);
        }
    }
}

fn tutorial_test_things() {
    use alloc::{rc::Rc, vec};
    use log::trace;

    // allocate a number on the heap
    let heap_value = Box::new(23);
    trace!("heap_value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    trace!("vec at {:p}", vec.as_slice());
    trace!(
        "Heap usage: {}\n",
        hakkero::allocator::ALLOCATOR.lock().used_heap()
    );

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    trace!(
        "Current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    trace!(
        "Heap usage: {}\n",
        hakkero::allocator::ALLOCATOR.lock().used_heap()
    );
    trace!("Dropping `reference_counted`");
    core::mem::drop(reference_counted);
    trace!(
        "Now, reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    trace!(
        "Heap usage: {}\n",
        hakkero::allocator::ALLOCATOR.lock().used_heap()
    );
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
    hakkero::test_panic_handler(info)
}
