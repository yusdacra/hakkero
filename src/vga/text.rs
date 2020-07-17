use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

use vga::colors::{Color16, TextModeColor};
use vga::writers::{ScreenCharacter, Text80x25, TextWriter};

use crate::woint;
#[cfg(test)]
use crate::{serial_print, serial_println};

pub type Color = Color16;
pub type WriterColor = TextModeColor;
type DefWriter = Text80x25;

lazy_static! {
    /// A global `Writer` instance that can be used for printing to the VGA text buffer.
    ///
    /// Used by the `print!` and `println!` macros.
    pub static ref WRITER: Mutex<VgaWriter<DefWriter>> = Mutex::new(VgaWriter::new(DefWriter::new()));
}

/// A writer type that allows writing ASCII bytes and strings using.
///
/// Wraps lines at `size.x`. Supports newline characters and implements the
/// `core::fmt::Write` trait.
pub struct VgaWriter<T: TextWriter + Send + Sync> {
    color: WriterColor,
    x_pos: usize,
    iw: T,
}

impl<T: TextWriter + Send + Sync> VgaWriter<T> {
    /// Create a new `VgaWriter` from the given `TextWriter`.
    pub fn new(iw: T) -> Self {
        Self {
            color: WriterColor::new(Color::White, Color::Black),
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
            if self.x_pos >= T::WIDTH {
                self.new_line();
            }
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
        ScreenCharacter::new(b' ', self.color)
    }
}

impl<T: TextWriter + Send + Sync> fmt::Write for VgaWriter<T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
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
        write!(writer, "{}", args).unwrap();
        writer.color = old_color;
    });
}

#[doc(hidden)]
/// Prints the given formatted string to the VGA text buffer through the global `WRITER` instance.
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    woint(|| write!(WRITER.lock(), "{}", args).unwrap());
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
    use vga::writers::Screen;

    serial_print!("test_println_output... ");

    let s = "Some test string that fits on a single line";
    woint(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.iw.read_character(i, DefWriter::HEIGHT - 2);
            assert_eq!(char::from(screen_char.get_character()), c);
        }
    });

    serial_println!("[ok]");
}
