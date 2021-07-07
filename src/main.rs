#![no_std]
#![no_main]
#![feature(custom_test_frameworks, naked_functions)]
#![test_runner(hakkero::test::runner)]
#![reexport_test_harness_main = "test_main"]

mod panic;

// Only compile alloc on systems where we have heap setup
#[cfg(target_arch = "x86_64")]
extern crate alloc;

#[cfg(target_arch = "x86_64")]
use hakkero::{
    arch::task::{handle_scancodes, DecodedKeyStream},
    task::{self, Executor, Task},
};

// NOTE: All supported architectures must have entry_point implemented!
hakkero::arch::entry_point!(kernel_main);

#[cfg(target_arch = "x86_64")]
fn kernel_main() -> ! {
    // We run tests before everything to avoid interference
    #[cfg(test)]
    test_main();

    heap_info();

    log::info!("Welcome to Hakkero OS!\n");
    Executor::new().spawn(Task::new(start_handlers())).run()
}

#[cfg(target_arch = "aarch64")]
fn kernel_main() -> ! {
    log::info!("Welcome to Hakkero OS!\n");

    // We run tests before everything to avoid interference
    #[cfg(test)]
    test_main();

    hakkero::arch::hang_cpu()
}

// Only compile on systems where we have heap setup
#[cfg(target_arch = "x86_64")]
fn heap_info() {
    use hakkero::allocator::*;
    use log::info;

    info!("Heap start: {}", HEAP_START);
    info!("Heap size : {}", HEAP_SIZE);
    info!("Heap usage: {}", ALLOCATOR.lock().used_heap());
}

#[cfg(target_arch = "x86_64")]
async fn start_handlers() {
    use pc_keyboard::DecodedKey;

    log::info!("starting services");
    task::spawn(Task::new(handle_scancodes())).unwrap();
    log::info!("handle keyboard scancodes started");
    task::spawn(Task::new(async {
        use futures_util::stream::StreamExt;

        let mut queue = DecodedKeyStream;

        while let Some(key) = queue.next().await {
            hakkero::print!(
                "{}",
                match key {
                    DecodedKey::Unicode(b) => b,
                    _ => ' ',
                }
            );
        }
    }))
    .unwrap();
}
