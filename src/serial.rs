pub use uart_16550::SerialPort;

/// Creates and initializes a serial port.
///
/// # Safety
/// The caller must guarantee that the given `base` really points to a serial port.
pub unsafe fn create_serial_port(base: u16) -> SerialPort {
    let mut serial_port = SerialPort::new(base);
    serial_port.init();
    serial_port
}

pub const QEMU_SP_ADDR: u16 = 0x03F8;

// TODO: Make this panic / return an error when not running in QEMU
/// Creates a serial port for use when running in QEMU.
pub fn create_qemu_sp() -> SerialPort {
    unsafe { create_serial_port(QEMU_SP_ADDR) }
}

#[doc(hidden)]
pub fn _print(serial_port: &mut SerialPort, args: core::fmt::Arguments) {
    use core::fmt::Write;

    crate::woint(|| write!(serial_port, "{}", args).expect("Printing to serial failed"));
}

/// Prints to the host through the serial interface.
#[macro_export]
macro_rules! serial_print {
    ($serial_port:expr, $($arg:tt)*) => {
        $crate::serial::_print($serial_port, format_args!($($arg)*));
    };
}

/// Prints to the host through the serial interface, appending a newline.
#[macro_export]
macro_rules! serial_println {
    ($serial_port:expr) => ($crate::serial_print!($serial_port, "\n"));
    ($serial_port:expr, $($arg:tt)*) => ($crate::serial_print!($serial_port, "{}\n", format_args!($($arg)*)));
}

use log::{Log, Metadata, Record};

pub struct SerialLogger {
    base: u16,
}

impl SerialLogger {
    /// Creates a new `SerialLogger`
    ///
    /// # Safety
    /// This is unsafe because the `log` function impl creates a `SerialPort` underneath.
    /// Therefore the caller must guarantee that the given `base` really points to a serial port.
    pub unsafe fn new(base: u16) -> Self {
        Self { base }
    }

    pub const fn new_qemu() -> Self {
        Self { base: QEMU_SP_ADDR }
    }
}

impl Log for SerialLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut sp = unsafe { create_serial_port(self.base) };

            crate::serial_println!(&mut sp, "[{:5}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
