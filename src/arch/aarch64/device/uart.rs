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
#[inline(always)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;

    write!(&mut UART, "{}", args).expect("Could not write to QEMU console!");
}

#[macro_export]
macro_rules! serial_print {
    ($($args:tt)*) => ($crate::arch::device::uart::_print(format_args!($($args)*)));
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::arch::device::uart::_print(format_args!("\n")));
    ($($arg:tt)*) => ($crate::arch::device::uart::_print(format_args!("{}\n", format_args!($($arg)*))));
}
