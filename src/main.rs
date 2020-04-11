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
    executor::{Executor, Spawner},
    keyboard, Task,
};
use hakkero::{print, println};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Initialize phase
    hakkero::init_heap(boot_info);
    hakkero::init();

    let mut executor = Executor::new();
    executor.spawn(Task::new(start(executor.spawner())));
    executor.run();
}

async fn start(spawner: Spawner) {
    use hakkero::vga_buffer::{change_writer_color as cwc, Color, WriterColor};

    // Show welcome text and run tests
    x86_64::instructions::interrupts::without_interrupts(|| {
        // Welcome text
        cwc(WriterColor::new(Color::LightRed, Color::Black));
        println!(
            "Welcome to, 
                                         __   __ 
     /  |      /    /                   /  | /   
    (___| ___ (    (     ___  ___  ___ (   |(___ 
    |   )|   )|___)|___)|___)|   )|   )|   )    )
    |  / |__/|| \\  | \\  |__  |    |__/ |__/  __/

(*very* powerful furnace OS)\n"
        );

        cwc(WriterColor::new(Color::LightBlue, Color::Black));
        println!("*cough* Testing...");
        tutorial_test_things();

        #[cfg(test)]
        test_main();

        cwc(WriterColor::new(Color::LightGreen, Color::Black));
        print!("Didn't crash. Am I doing something right?");
        cwc(WriterColor::new(Color::White, Color::Black));
        println!();
    });

    spawner.spawn(Task::new(start_handlers(spawner.clone())));
}

async fn start_handlers(spawner: Spawner) {
    spawner.spawn(Task::new(keyboard::handle_scancodes()));
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
    println!("{}", info);
    hakkero::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    hakkero::test_panic_handler(info)
}
