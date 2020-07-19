use vga::colors::{Color16, TextModeColor};
use vga::writers::{ScreenCharacter, TextWriter};

use crate::woint;
#[cfg(test)]
use crate::{serial_print, serial_println};

pub type VgaWriterColor = TextModeColor;

/// Convenience alias.
pub trait Writer = TextWriter + Send + Sync;

/// A writer type that allows writing ASCII bytes and strings using.
///
/// Wraps lines at `size.x`. Supports newline characters and implements the
/// `core::fmt::Write` trait.
pub struct VgaWriter<T: Writer> {
    color: VgaWriterColor,
    def_color: VgaWriterColor,
    x_pos: usize,
    iw: T,
}

impl<T: Writer> VgaWriter<T> {
    /// Create a new `VgaWriter` from the given `TextWriter`.
    pub fn new(iw: T) -> Self {
        let color = VgaWriterColor::new(Color16::White, Color16::Black);
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
            self.iw.write_character(x, y, self.blank_char());
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

    #[allow(dead_code)]
    /// Clears the screen by filling it with a blank character.
    fn clear_screen(&self) {
        self.iw.fill_screen(self.blank_char());
    }

    /// Returns a blank character ' ' with current color.
    fn blank_char(&self) -> ScreenCharacter {
        ScreenCharacter::new(b' ', self.def_color)
    }
}

impl<T: Writer> core::fmt::Write for VgaWriter<T> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

use alloc::sync::Arc;
use log::{Log, Metadata, Record};
use spin::Mutex;

// TODO: Remove the `Arc` (so, don't allocate) so that we can use it before allocator initialization.
pub struct VgaLogger<T: Writer> {
    ivw: Arc<Mutex<VgaWriter<T>>>,
}

impl<T: Writer> VgaLogger<T> {
    pub fn new(ivw: Arc<Mutex<VgaWriter<T>>>) -> Self {
        Self { ivw }
    }
}

impl<T: Writer> Log for VgaLogger<T> {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            use log::Level;

            let color = match record.level() {
                Level::Error => VgaWriterColor::new(Color16::Black, Color16::Red),
                Level::Warn => VgaWriterColor::new(Color16::Yellow, Color16::Black),
                Level::Info => VgaWriterColor::new(Color16::Blue, Color16::Black),
                _ => VgaWriterColor::new(Color16::White, Color16::Black),
            };
            crate::println_colored!(
                &mut self.ivw.lock(),
                color,
                "[{:5}] {}",
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

/// Like the `print!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! print {
    ($writer:expr, $($arg:tt)*) => ($crate::vga::writer::_print($writer, format_args!($($arg)*)));
}

/// Like the `println!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! println {
    ($writer:expr) => ($crate::print!($writer, "\n"));
    ($writer:expr, $($arg:tt)*) => ($crate::print!($writer, "{}\n", format_args!($($arg)*)));
}

/// Prints text to VGA buffer colored with given color.
#[macro_export]
macro_rules! print_colored {
    ($writer:expr, $color:expr, $($arg:tt)*) => ($crate::vga::writer::_print_colored($writer, $color, format_args!($($arg)*)));
}

/// Prints text to VGA buffer colored with given color, but with a new line at the end.
#[macro_export]
macro_rules! println_colored {
    ($writer:expr, $color:expr) => ($crate::print_colored!($writer, $color, "\n"));
    ($writer:expr, $color:expr, $($arg:tt)*) => ($crate::print_colored!($writer, $color, "{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
/// Writes the text colored in given `VgaWriterColor` to given `VgaWriter`. Changes the colors to old colors when printing finishes.
pub fn _print_colored<T: Writer>(
    writer: &mut VgaWriter<T>,
    color: VgaWriterColor,
    args: core::fmt::Arguments,
) {
    use core::fmt::Write;

    woint(|| {
        let old_color = writer.color;
        writer.color = color;
        write!(writer, "{}", args).expect("failed to write to vga buffer (literally how)");
        writer.color = old_color;
    });
}

#[doc(hidden)]
/// Writes the given formatted string to the VGA text buffer through the given `VgaWriter`.
pub fn _print<T: Writer>(writer: &mut VgaWriter<T>, args: core::fmt::Arguments) {
    use core::fmt::Write;

    woint(|| write!(writer, "{}", args).expect("failed to write to vga buffer (literally how)"));
}

#[cfg(test)]
use vga::writers::Text80x25;

#[test_case]
fn test_println_simple(sp: &mut crate::serial::SerialPort) {
    let mut writer = VgaWriter::new(Text80x25::new());

    serial_print!(sp, "test_println... ");
    println!(&mut writer, "test_println_simple output");
    serial_println!(sp, "[ok]");
}

#[test_case]
fn test_println_many(sp: &mut crate::serial::SerialPort) {
    let mut writer = VgaWriter::new(Text80x25::new());

    serial_print!(sp, "test_println_many... ");
    for _ in 0..200 {
        println!(&mut writer, "test_println_many output");
    }
    serial_println!(sp, "[ok]");
}

#[test_case]
fn test_println_output(sp: &mut crate::serial::SerialPort) {
    let mut writer = VgaWriter::new(Text80x25::new());

    serial_print!(sp, "test_println_output... ");

    let s = "1234567890";
    woint(|| {
        println!(&mut writer, "\n{}", s);
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.iw.read_character(i, Text80x25::HEIGHT - 2);
            assert_eq!(screen_char.get_character(), c as u8);
        }
    });

    serial_println!(sp, "[ok]");
}

#[test_case]
fn test_println_fit_line_output(sp: &mut crate::serial::SerialPort) {
    let mut writer = VgaWriter::new(Text80x25::new());

    serial_print!(sp, "test_println_fit_line_output... ");

    for _ in 0..Text80x25::WIDTH {
        print!(&mut writer, "a");
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

    serial_println!(sp, "[ok]");
}

#[test_case]
fn test_clear_screen(sp: &mut crate::serial::SerialPort) {
    let writer = VgaWriter::new(Text80x25::new());

    serial_print!(sp, "test_clear_screen... ");

    writer
        .get_iw()
        .fill_screen(ScreenCharacter::new(b'a', writer.def_color));
    writer.clear_screen();
    for y in 0..Text80x25::HEIGHT {
        for x in 0..Text80x25::WIDTH {
            assert_eq!(writer.get_iw().read_character(x, y).get_character(), b' ');
        }
    }

    serial_println!(sp, "[ok]");
}
