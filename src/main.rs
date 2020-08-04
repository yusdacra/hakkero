#![no_std]
#![no_main]
#![feature(custom_test_frameworks, naked_functions)]
#![test_runner(hakkero::test::runner)]
#![reexport_test_harness_main = "test_main"]

mod panic;

// NOTE: All supported architectures must have entry_point implemented!
use hakkero::entry_point;

// Only compile alloc on systems where we have heap setup
#[cfg(target_arch = "x86_64")]
extern crate alloc;

#[cfg(target_arch = "x86_64")]
use hakkero::{
    arch::{
        device::vga::Readline,
        task::{handle_scancodes, DecodedKeyStream},
    },
    task::{spawn_task, Executor, Task},
};

entry_point!(kernel_main);

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
    // We run tests before everything to avoid interference
    #[cfg(test)]
    test_main();

    hakkero::serial_println!("Welcome to Hakkero OS!\n");
    // log::info!("Welcome to Hakkero OS!\n");

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
    spawn_task(Task::new(handle_scancodes())).unwrap();
    spawn_task(Task::new(async {
        use futures_util::stream::StreamExt;

        let mut queue = DecodedKeyStream;
        let mut rl = Readline::new();

        while let Some(key) = queue.next().await {
            if let Some(s) = rl.handle_key(key) {
                hakkero::println!("{}", s);
            }
        }
    }))
    .unwrap();
}
