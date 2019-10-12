#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(asm)]
#![test_runner(hakkero::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader::{entry_point, BootInfo};
extern crate alloc;

use hakkero::{hlt_loop, println, print};
use core::panic::PanicInfo;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
	use hakkero::vga_buffer::{self, Color};

	// Initialize things
	hakkero::init_heap(boot_info);
	hakkero::init();

	// Run these without no interrupts
	x86_64::instructions::interrupts::without_interrupts(|| {
		// Welcome text
		vga_buffer::change_writer_color(Color::LightRed, Color::Black);
		println!(
            "Welcome to, 
                                         __   __ 
     /  |      /    /                   /  | /   
    (___| ___ (    (     ___  ___  ___ (   |(___ 
    |   )|   )|___)|___)|___)|   )|   )|   )    )
    |  / |__/|| \\  | \\  |__  |    |__/ |__/  __/

(*very* powerful furnace OS)\n"
		);

		vga_buffer::change_writer_color(Color::LightBlue, Color::Black);
		println!("*cough* Testing...");
		tutorial_test_things();

		#[cfg(test)]
		test_main();

		vga_buffer::change_writer_color(Color::LightGreen, Color::Black);
		println!("Didn't crash. Am I doing something right?");
	});

	// Shell time bois
	vga_buffer::change_writer_color(Color::LightGray, Color::Black);
	print!("\n:> ");

	loop {
		x86_64::instructions::interrupts::without_interrupts(|| {
			if let Some(data) = hakkero::readline::RL.lock().retrieve_data() {
				let data = data.trim();
				// Shutdown for QEMU
				if data == "shutdown" {
					// Replace with the actual shutdown assembly / code
					unsafe {
						x86_64::instructions::port::Port::<u16>::new(0x604).write(0x2000);
					}
				} else if data == "reboot" {
					unsafe {
						x86_64::instructions::port::Port::<u8>::new(0x64).write(0xFE);
					}
				} else if data == "clear" {
					for row in 0..=24 {
						vga_buffer::clear_row(row);
					}
				} else if data == "Hello," {
					println!("world!");
				} else if !data.is_empty() {
					println!("no such command: {}", data);
				}
				print!(":> ");
			}
		});
		x86_64::instructions::hlt();
	}
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
	hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
	hakkero::test_panic_handler(info)
}
