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

lazy_static! {
    /// A global `Writer` instance that can be used for printing to the VGA text buffer.
    ///
    /// Used by the `print!` and `println!` macros.
    pub static ref WRITER: Mutex<VgaWriter<Text80x25>> = Mutex::new(VgaWriter {
        color: WriterColor::new(Color::White, Color::Black),
        geo: BufferGeometry::new(80, 25),
        iw: Text80x25::new(),
    });
}

/// Stores buffer size and current cursor position information.
pub struct BufferGeometry {
    x_size: usize,
    y_size: usize,
    x_pos: usize,
    y_pos: usize,
}

impl BufferGeometry {
    pub fn new(x_size: usize, y_size: usize) -> Self {
        Self {
            x_size,
            y_size,
            x_pos: 0,
            y_pos: 0,
        }
    }

    /// Set position (`x_pos` for x axis and `y_pos` for y axis).
    /// Clamps to `x_size` for `x` and `y_size` for `y`.
    pub fn set_pos(&mut self, x: usize, y: usize) {
        self.x_pos = if x > self.x_size { self.x_size } else { x };
        self.y_pos = if y > self.y_size { self.y_size } else { y };
    }

    /// Set x axis position.
    pub fn set_x(&mut self, to: usize) {
        self.set_pos(to, self.y_pos)
    }

    /// Set y axis position.
    pub fn set_y(&mut self, to: usize) {
        self.set_pos(self.x_pos, to)
    }

    /// Offset the position by `x` for `x_pos` and `y` for `y_pos`.
    pub fn offset_pos(&mut self, x: isize, y: isize) {
        self.set_pos(
            (self.x_pos as isize + x) as usize,
            (self.y_pos as isize + y) as usize,
        )
    }

    /// Offset x axis.
    pub fn offset_x(&mut self, by: isize) {
        self.offset_pos(by, 0)
    }

    /// Offset y axis.
    pub fn offset_y(&mut self, by: isize) {
        self.offset_pos(0, by)
    }
}

/// A writer type that allows writing ASCII bytes and strings using.
///
/// Wraps lines at `size.x`. Supports newline characters and implements the
/// `core::fmt::Write` trait.
pub struct VgaWriter<T: TextWriter> {
    color: WriterColor,
    geo: BufferGeometry,
    iw: T,
}

impl<T: TextWriter> VgaWriter<T> {
    /// Changes internal `TextWriter` and updates graphics mode.
    pub fn set_writer(&mut self, writer: T) {
        self.iw = writer;
        self.iw.set_mode();
    }

    /// Mutable access to inner `BufferGeometry`.
    pub fn get_mut_geo(&mut self) -> &mut BufferGeometry {
        &mut self.geo
    }

    /// Access to inner `TextWriter`.
    pub fn get_iw(&self) -> &T {
        &self.iw
    }

    /// Writes an ASCII byte to the buffer at current position.
    pub fn write_byte(&mut self, byte: u8) {
        let screen_char = ScreenCharacter::new(byte, self.color);
        self.iw
            .write_character(self.geo.x_pos, self.geo.y_pos, screen_char);
    }

    /// Writes the given ASCII string to the buffer.
    ///
    /// Wraps lines at `size.x`. Supports the `\n` newline character. Does **not**
    /// support strings with non-ASCII characters, since they can't be printed in the VGA text
    /// mode.
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            if self.geo.x_pos >= self.geo.x_size {
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
            self.geo.offset_x(1);
        }
        // Update cursor position
        self.iw.set_cursor_position(self.geo.x_pos, self.geo.y_pos);
    }

    /// Shifts all lines one line up and clears the last line.
    fn new_line(&mut self) {
        for y in 1..self.geo.y_size {
            for x in 0..self.geo.x_size {
                let character = self.iw.read_character(x, y);
                self.iw.write_character(x, y - 1, character);
            }
        }
        self.clear_line(self.geo.y_size - 1);
        self.geo.set_pos(0, self.geo.y_size - 1);
    }

    /// Clears a line by overwriting it with a blank character.
    fn clear_line(&mut self, y: usize) {
        let blank = ScreenCharacter::new(b' ', self.color);
        for x in 0..self.geo.x_size {
            self.iw.write_character(x, y, blank);
        }
    }

    /// Clears the screen by filling it with a blank character.
    fn clear_screen(&mut self) {
        for y in 0..self.geo.y_size {
            self.clear_line(y);
        }
    }
}

impl<T: TextWriter> fmt::Write for VgaWriter<T> {
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

    let mut writer = WRITER.lock();
    let old_color = writer.color;
    writer.color = color;
    woint(|| writer.write_fmt(args).unwrap());
    writer.color = old_color;
}

#[doc(hidden)]
/// Prints the given formatted string to the VGA text buffer through the global `WRITER` instance.
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;

    woint(|| WRITER.lock().write_fmt(args).unwrap());
}

/// Changes the global `WRITER` instance `WriterColor` to the given `WriterColor`.
pub fn change_writer_color(color: WriterColor) {
    WRITER.lock().color = color;
}

/// Clears the screen using the global `WRITER` instance.
pub fn clear_screen() {
    woint(|| WRITER.lock().clear_screen());
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

    let s = "Some test string that fits on a single line";
    woint(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.iw.read_character(i, writer.geo.y_size - 2);
            assert_eq!(char::from(screen_char.get_character()), c);
        }
    });

    serial_println!("[ok]");
}
