use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::InterruptStackFrame;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub fn init() {
    unsafe {
        PICS.lock().initialize();
    };
    x86_64::instructions::interrupts::enable();
}

/// Convenience function to notify the end of an interrupt.
pub fn send_eoi(int_index: InterruptIndex) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(int_index.as_u8());
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    pub const fn as_usize(self) -> usize {
        self.as_u8() as usize
    }
}

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    send_eoi(InterruptIndex::Timer);
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = x86_64::instructions::port::PortReadOnly::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    super::super::task::keyboard::add_scancode(scancode);

    send_eoi(InterruptIndex::Keyboard);
}
