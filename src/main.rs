#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(asm)]
#![test_runner(hakkero::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
extern crate alloc;

use core::panic::PanicInfo;
use hakkero::task::{
    executor::{spawn_task, Executor},
    keyboard, Task,
};
use hakkero::vga::{Logger, Readline, Writer};
use spin::{Mutex, Once};
use vga::writers::Text80x25;

lazy_static::lazy_static! {
    static ref WRITER: Mutex<Writer<Text80x25>> = Mutex::new(Writer::default());
}

// NOTE: lazy_static doesn't work with set_logger for some reason (weird "`Log` not implemented for `LOGGER`" error) so we use `Once`
static LOGGER: Once<Logger<Text80x25>> = Once::new();

entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Set up logger
    LOGGER.call_once(|| Logger::new(&WRITER));

    // TODO: Look into using `set_logger_boxed` (fork `log` and use `alloc` instead of `std` in feature? (maybe even make a PR for that?))
    log::set_logger(LOGGER.r#try().unwrap()).expect("Could not setup logger.");
    log::set_max_level(log::LevelFilter::Trace);

    // Initialize phase
    hakkero::init();
    hakkero::init_heap(boot_info);

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
    info!("Heap size: {}\n", hakkero::allocator::HEAP_SIZE);
}

async fn start_handlers() {
    spawn_task(Task::new(keyboard::handle_scancodes()));
    spawn_task(Task::new(handle_decoded_keys()));
}

async fn handle_decoded_keys() {
    use futures_util::stream::StreamExt;

    let mut queue = hakkero::task::keyboard::DecodedKeyStream;
    let mut rl = Readline::default();

    while let Some(key) = queue.next().await {
        if let Some(s) = rl.handle_key(key) {
            log::trace!(
                "rl output: {}\n",
                s.into_iter()
                    .map(|b| b as char)
                    .collect::<alloc::string::String>()
            );
        }
    }
}

fn tutorial_test_things() {
    use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};
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

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    trace!(
        "Current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    trace!("Dropping `reference_counted`");
    core::mem::drop(reference_counted);
    trace!(
        "Now, reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);
    hakkero::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    hakkero::test_panic_handler(info)
}
