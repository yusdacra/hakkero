pub mod readline;

use crate::arch::x86_64::woint;
use spin::Mutex;
use vga::{
    colors::{Color16, TextModeColor},
    writers::{ScreenCharacter, Text80x25, TextWriter},
};

lazy_static::lazy_static! {
    static ref WRITER: Mutex<Writer<Text80x25>> = Mutex::new(Writer::default());
}

/// A writer type that allows writing ASCII bytes and strings using.
///
/// Wraps lines at `size.x`. Supports newline characters and implements the
/// `core::fmt::Write` trait.
pub struct Writer<T: TextWriter> {
    color: TextModeColor,
    def_color: TextModeColor,
    x_pos: usize,
    iw: T,
}

impl<T: TextWriter> Writer<T> {
    /// Create a new `Writer` from the given `TextWriter`.
    pub fn new(iw: T) -> Self {
        let color = TextModeColor::new(Color16::White, Color16::Black);
        iw.set_mode();
        Self {
            color,
            def_color: color,
            x_pos: 0,
            iw,
        }
    }

    /// Access to inner `TextWriter`.
    pub fn get_iw(&self) -> &T {
        &self.iw
    }

    /// Writes an ASCII byte to the buffer at current position.
    fn write_byte(&self, byte: u8) {
        self.iw.write_character(
            self.x_pos,
            T::HEIGHT - 1,
            ScreenCharacter::new(byte, self.color),
        );
    }

    /// Writes the given ASCII string to the buffer.
    ///
    /// Wraps lines at `size.x`. Supports the `\n` newline character. Does **not**
    /// support strings with non-ASCII characters, since they can't be printed in the VGA text
    /// mode.
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte
                0x20..=0x7e => self.write_byte(byte),
                // newline
                b'\n' => {
                    self.new_line();
                    continue; // here to skip x offset
                }
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
            self.x_pos += 1;
            if self.x_pos >= T::WIDTH {
                self.new_line();
            }
        }
        // Update cursor position
        self.iw.set_cursor_position(self.x_pos, T::HEIGHT - 1);
    }

    /// Clears a line by overwriting it with a blank character.
    fn clear_line(&self, y: usize) {
        for x in 0..T::WIDTH {
            self.iw
                .write_character(x, y, ScreenCharacter::new(b' ', self.def_color));
        }
    }

    /// Clears the last line and shifts all lines one line upwards.
    fn new_line(&mut self) {
        for y in 1..T::HEIGHT {
            for x in 0..T::WIDTH {
                let character = self.iw.read_character(x, y);
                self.iw.write_character(x, y - 1, character);
            }
        }
        self.clear_line(T::HEIGHT - 1);
        self.x_pos = 0;
    }
}

impl<T: TextWriter> core::fmt::Write for Writer<T> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

impl Default for Writer<Text80x25> {
    fn default() -> Self {
        Self::new(Text80x25::new())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::arch::x86_64::device::vga::text::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print_colored {
    ($color:expr, $($arg:tt)*) => ($crate::arch::x86_64::device::vga::text::_print_colored($color, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println_colored {
    ($color:expr) => ($crate::print_colored!($color, "\n"));
    ($color:expr, $($arg:tt)*) => ($crate::print_colored!($color, "{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print_colored(color: TextModeColor, args: core::fmt::Arguments) {
    #[cfg(not(test))]
    use core::fmt::Write;

    woint(|| {
        let mut writer = if let Some(g) = WRITER.try_lock() {
            g
        } else {
            return;
        };
        let old_color = writer.color;
        writer.color = color;
        write!(&mut writer, "{}", args).expect("failed to write to vga buffer (literally how)");
        writer.color = old_color;
    });
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    #[cfg(not(test))]
    use core::fmt::Write;

    woint(|| {
        write!(
            &mut if let Some(g) = WRITER.try_lock() {
                g
            } else {
                return;
            },
            "{}",
            args
        )
        .expect("failed to write to vga buffer (literally how)")
    });
}

// TESTS

#[cfg(test)]
use {
    crate::{serial_print, serial_println},
    core::fmt::Write,
    vga::writers::Screen,
};

#[test_case]
fn test_println_simple() {
    let mut writer = Writer::default();

    serial_print!("test_println... ");
    writeln!(&mut writer, "test_println_simple output").unwrap();
    serial_println!("[ok]");
}

#[test_case]
fn test_println_many() {
    let mut writer = Writer::default();

    serial_print!("test_println_many... ");
    for _ in 0..200 {
        writeln!(&mut writer, "test_println_many output").unwrap();
    }
    serial_println!("[ok]");
}

#[test_case]
fn test_println_output() {
    let mut writer = Writer::default();

    serial_print!("test_println_output... ");

    let s = "1234567890";
    writeln!(&mut writer, "\n{}", s).unwrap();
    for (i, c) in s.chars().enumerate() {
        let screen_char = writer.iw.read_character(i, Text80x25::HEIGHT - 2);
        assert_eq!(screen_char.get_character(), c as u8);
    }

    serial_println!("[ok]");
}

#[test_case]
fn test_println_fit_line_output() {
    let mut writer = Writer::default();

    serial_print!("test_println_fit_line_output... ");

    for _ in 0..Text80x25::WIDTH {
        write!(&mut writer, "a").unwrap();
    }
    assert_eq!(
        char::from(
            writer
                .get_iw()
                .read_character(writer.x_pos, Text80x25::HEIGHT - 1)
                .get_character()
        ),
        ' '
    );

    serial_println!("[ok]");
}
