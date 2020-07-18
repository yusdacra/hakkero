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
use hakkero::{println, println_colored};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    log::set_logger(&hakkero::vga::text::VgaLogger).expect("Could not setup logger.");
    log::set_max_level(log::LevelFilter::Trace);

    // Initialize phase
    hakkero::init_heap(boot_info);
    hakkero::init();

    welcome_message();

    let mut executor = Executor::new();
    executor.spawn(Task::new(start_handlers()));
    executor.run();
}

fn welcome_message() {
    use hakkero::vga::text::{change_writer_color as cwc, Color, WriterColor};
    cwc(WriterColor::new(Color::White, Color::Black));

    // Show welcome text and run tests
    hakkero::woint(|| {
        // Welcome text
        println_colored!(
            WriterColor::new(Color::LightRed, Color::Black),
            "Welcome to, 
                                         __   __ 
     /  |      /    /                   /  | /   
    (___| ___ (    (     ___  ___  ___ (   |(___ 
    |   )|   )|___)|___)|___)|   )|   )|   )    )
    |  / |__/|| \\  | \\  |__  |    |__/ |__/  __/

(*very* powerful furnace OS)\n",
        );

        println_colored!(
            WriterColor::new(Color::LightBlue, Color::Black),
            "*cough* Testing..."
        );
        tutorial_test_things();

        #[cfg(test)]
        test_main();

        println_colored!(
            WriterColor::new(Color::LightGreen, Color::Black),
            "Didn't crash. Am I doing something right?"
        );
    });
}

async fn start_handlers() {
    spawn_task(Task::new(keyboard::handle_scancodes()));
}

fn tutorial_test_things() {
    use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};

    // allocate a number on the heap
    let heap_value = Box::new(23);
    println!("heap_value at {:p}", heap_value);

    // create a dynamically sized vector
    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    // create a reference counted vector -> will be freed when count reaches 0
    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!(
        "Current reference count is {}",
        Rc::strong_count(&cloned_reference)
    );
    println!("Dropping `reference_counted`");
    core::mem::drop(reference_counted);
    println!(
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
