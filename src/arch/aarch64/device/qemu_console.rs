use core::fmt;

pub struct QemuConsole;

impl fmt::Write for QemuConsole {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            unsafe {
                core::ptr::write_volatile(0x3F20_1000 as *mut u8, c as u8);
            }
        }

        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;

    write!(&mut QemuConsole, "{}", args).expect("Could not write to QEMU console!");
}

#[macro_export]
macro_rules! console_print {
    ($($args:tt)*) => ($crate::arch::aarch64::device::qemu_console::_print(format_args!($($args)*)));
}

#[macro_export]
macro_rules! console_println {
    () => ($crate::arch::aarch64::device::qemu_console::_print(format_args!("\n")));
    ($($arg:tt)*) => ($crate::arch::aarch64::device::qemu_console::_print(format_args!("{}\n", format_args!($($arg)*))));
}
