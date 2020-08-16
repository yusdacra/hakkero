use crate::arch::x86_64::woint;
use core::sync::atomic::{AtomicBool, Ordering};
use spinning_top::Spinlock;
use uart_16550::SerialPort;

pub fn init() {
    log::trace!("Initializing UART 16550 serial port...");
    woint(|| {
        COM1.lock().init();
        INITIALIZED.store(true, Ordering::SeqCst);
    });
    log::info!("Successfully initialized UART 16550 serial port!");
}

pub const COM1_ADDR: u16 = 0x03F8;

static INITIALIZED: AtomicBool = AtomicBool::new(false);
static COM1: Spinlock<SerialPort> = Spinlock::new(unsafe { SerialPort::new(COM1_ADDR) });

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;

    if !INITIALIZED.load(Ordering::SeqCst) {
        return;
    }

    woint(|| {
        write!(
            &mut if let Some(g) = COM1.try_lock() {
                g
            } else {
                return;
            },
            "{}",
            args
        )
        .expect("Printing to serial failed")
    });
}

pub macro serial_print($($arg:tt)*) {
    $crate::arch::device::uart16550::_print(format_args!($($arg)*));
}

pub macro serial_println($($arg:tt)*) {
    $crate::arch::device::uart16550::_print(format_args!("{}\n", format_args!($($arg)*)));
}
