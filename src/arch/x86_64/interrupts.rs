use super::{
    device::pic8259::{self, keyboard_interrupt_handler, timer_interrupt_handler},
    gdt,
};
use crate::misc::Once;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

static IDT: Once<InterruptDescriptorTable> = Once::new();

/// Initializes the IDT.
///
/// # Safety
/// Must only be called once.
pub unsafe fn init_idt() {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.page_fault.set_handler_fn(page_fault_handler);
    idt.double_fault
        .set_handler_fn(double_fault_handler)
        .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
    idt[pic8259::InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
    idt[pic8259::InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
    IDT.try_init(idt).load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: &mut InterruptStackFrame) {
    log::info!("EXPECTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    panic!(
        "\
EXPECTION: PAGE FAULT
Accessed Address: {:?}
Error Code: {:?}
{:#?}
        ",
        x86_64::registers::control::Cr2::read(),
        error_code,
        stack_frame,
    );
}

// TESTS

#[cfg(test)]
use crate::{serial_print, serial_println};

#[test_case]
fn test_breakpoint_exception() {
    serial_print!("test_breakpoint_exception... ");
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
    serial_println!("[ok]");
}
