use crate::arch::x86_64::interrupts::InterruptIndex;
use pic8259_simple::ChainedPics;
use spin::Mutex;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub fn init() {
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

/// Convenience function to notify the end of an interrupt.
pub fn send_eoi(int_index: InterruptIndex) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(int_index.as_u8());
    }
}
