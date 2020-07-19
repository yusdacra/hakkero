use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

use vga::colors::{Color16, TextModeColor};
use vga::writers::{ScreenCharacter, TextWriter};

use crate::woint;
#[cfg(test)]
use crate::{serial_print, serial_println};

pub type Color = Color16;
pub type WriterColor = TextModeColor;
type DefWriter = vga::writers::Text80x25;

lazy_static! {
    /// A global `Writer` instance that can be used for printing to the VGA text buffer.
    ///
    /// Used by the `print!` and `println!` macros.
    pub static ref WRITER: Mutex<VgaWriter<DefWriter>> = Mutex::new(VgaWriter::new(DefWriter::new()));
}

/// Convenience alias.
pub trait Writer = TextWriter + Send + Sync;

/// A writer type that allows writing ASCII bytes and strings using.
///
/// Wraps lines at `size.x`. Supports newline characters and implements the
/// `core::fmt::Write` trait.
pub struct VgaWriter<T: Writer> {
    color: WriterColor,
    def_color: WriterColor,
    x_pos: usize,
    iw: T,
}

impl<T: Writer> VgaWriter<T> {
    /// Create a new `VgaWriter` from the given `TextWriter`.
    pub fn new(iw: T) -> Self {
        let color = WriterColor::new(Color::White, Color::Black);
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

impl<T: Writer> fmt::Write for VgaWriter<T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

use log::{Log, Metadata, Record};

pub struct VgaLogger;

impl Log for VgaLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            use log::Level;

            let color = match record.level() {
                Level::Error => WriterColor::new(Color::Black, Color::Red),
                Level::Warn => WriterColor::new(Color::Yellow, Color::Black),
                Level::Info => WriterColor::new(Color::Blue, Color::Black),
                _ => WriterColor::new(Color::White, Color::Black),
            };
            crate::println_colored!(color, "[{:5}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

/// Like the `print!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::text::_print(format_args!($($arg)*)));
}

/// Like the `println!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Prints text to VGA buffer colored with given color.
#[macro_export]
macro_rules! print_colored {
    ($color:expr, $($arg:tt)*) => ($crate::vga::text::_print_colored(format_args!($($arg)*), $color));
}

/// Prints text to VGA buffer colored with given color, but with a new line at the end.
#[macro_export]
macro_rules! println_colored {
    ($color:expr) => ($crate::print_colored!($color, "\n"));
    ($color:expr, $($arg:tt)*) => ($crate::print_colored!($color, "{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
/// Prints the text colored in given `WriterColor`. Changes the colors to old colors when printing finishes.
pub fn _print_colored(args: fmt::Arguments, color: WriterColor) {
    use core::fmt::Write;

    woint(|| {
        let mut writer = WRITER.lock();
        let old_color = writer.color;
        writer.color = color;
        write!(writer, "{}", args).expect("failed to write to vga buffer (literally how)");
        writer.color = old_color;
    });
}

#[doc(hidden)]
/// Prints the given formatted string to the VGA text buffer through the global `WRITER` instance.
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    woint(|| {
        write!(WRITER.lock(), "{}", args).expect("failed to write to vga buffer (literally how)")
    });
}

/// Changes the global `WRITER` instance `WriterColor` to the given `WriterColor`.
pub fn change_writer_color(color: WriterColor) {
    woint(|| {
        WRITER.lock().color = color;
    })
}

#[test_case]
fn test_println_simple() {
    serial_print!("test_println... ");
    println!("test_println_simple output");
    serial_println!("[ok]");
}

#[test_case]
fn test_println_many() {
    serial_print!("test_println_many... ");
    for _ in 0..200 {
        println!("test_println_many output");
    }
    serial_println!("[ok]");
}

#[test_case]
fn test_println_output() {
    use core::fmt::Write;

    serial_print!("test_println_output... ");

    let s = "1234567890";
    woint(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.iw.read_character(i, DefWriter::HEIGHT - 2);
            assert_eq!(screen_char.get_character(), c as u8);
        }
    });

    serial_println!("[ok]");
}

#[test_case]
fn test_println_fit_line_output() {
    serial_print!("test_println_fit_line_output... ");

    for _ in 0..DefWriter::WIDTH {
        print!("a");
    }
    let writer = WRITER.lock();
    assert_eq!(
        char::from(
            writer
                .get_iw()
                .read_character(writer.x_pos, DefWriter::HEIGHT - 1)
                .get_character()
        ),
        ' '
    );

    serial_println!("[ok]");
}

#[test_case]
fn test_clear_screen() {
    serial_print!("test_clear_screen... ");

    let writer = WRITER.lock();
    writer
        .get_iw()
        .fill_screen(ScreenCharacter::new(b'a', writer.def_color));
    writer.clear_screen();
    for y in 0..DefWriter::HEIGHT {
        for x in 0..DefWriter::WIDTH {
            assert_eq!(writer.get_iw().read_character(x, y).get_character(), b' ');
        }
    }

    serial_println!("[ok]");
}