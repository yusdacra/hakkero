#![cfg(target_arch = "x86_64")]
#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use hakkero::{
    arch::gdt,
    misc::Once,
    serial_print, serial_println,
    test::{self, exit_qemu, QemuExitCode},
};
use x86_64::structures::idt::InterruptDescriptorTable;
use x86_64::structures::idt::InterruptStackFrame;

#[no_mangle]
#[allow(unreachable_code)]
unsafe extern "C" fn _start() -> ! {
    serial_print!("stack_overflow... ");
    #[cfg(not(debug_assertions))]
    {
        serial_println!("[ok]");
        exit_qemu(QemuExitCode::Success);
        loop {}
    }

    gdt::init();
    init_test_idt();

    stack_overflow();

    panic!("Execution continued after stack overflow");
}

#[allow(unconditional_recursion)]
fn stack_overflow() {
    stack_overflow(); // for each recursion, the return address is pushed
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test::panic_handler(info)
}

// Setup IDT
static TEST_IDT: Once<InterruptDescriptorTable> = Once::new();

pub unsafe fn init_test_idt() {
    let mut idt = InterruptDescriptorTable::new();
    idt.double_fault
        .set_handler_fn(test_double_fault_handler)
        .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    TEST_IDT.try_init(idt).load();
}

extern "x86-interrupt" fn test_double_fault_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}
