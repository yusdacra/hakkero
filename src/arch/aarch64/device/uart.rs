use super::super::board::UART_ADDR;
use core::fmt;

pub struct UART;

impl fmt::Write for UART {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            unsafe {
                core::ptr::write_volatile(UART_ADDR as *mut u8, b as u8);
            }
        }

        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;

    write!(&mut UART, "{}", args).expect("Could not write to QEMU console!");
}

pub macro serial_print($($args:tt)*) {
     $crate::arch::device::uart::_print(format_args!($($args)*));
}

pub macro serial_println($($arg:tt)*) {
     $crate::arch::device::uart::_print(format_args!("{}\n", format_args!($($arg)*)));
}
