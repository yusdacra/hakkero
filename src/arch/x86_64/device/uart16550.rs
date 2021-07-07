use crate::arch::x86_64::woint;
use spin::Mutex;
use uart_16550::SerialPort;

pub fn init() {
    woint(|| COM1.lock().init());
}

pub const COM1_ADDR: u16 = 0x03F8;

static COM1: Mutex<SerialPort> = Mutex::new(unsafe { SerialPort::new(COM1_ADDR) });

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;

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

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => ($crate::arch::device::uart16550::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::arch::device::uart16550::_print(format_args!("\n")));
    ($($arg:tt)*) => ($crate::arch::device::uart16550::_print(format_args!("{}\n", format_args!($($arg)*))));
}
