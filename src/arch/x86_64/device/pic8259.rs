use pic8259_simple::ChainedPics;
use spinning_top::Spinlock;
use x86_64::structures::idt::InterruptStackFrame;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
static PICS: Spinlock<ChainedPics> =
    Spinlock::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub fn init() {
    log::trace!("Initalizing PIC 8529...");
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
    log::info!("Successfully initialized PIC 8529!");
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
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    send_eoi(InterruptIndex::Timer);
}

pub extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    let mut port = x86_64::instructions::port::PortReadOnly::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    send_eoi(InterruptIndex::Keyboard);
}
