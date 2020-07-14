use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

use vga::colors::{Color16, TextModeColor};
use vga::writers::{ScreenCharacter, Text80x25, TextWriter};

#[cfg(test)]
use crate::{serial_print, serial_println};

pub type Color = Color16;
pub type WriterColor = TextModeColor;

lazy_static! {
    /// A global `Writer` instance that can be used for printing to the VGA text buffer.
    ///
    /// Used by the `print!` and `println!` macros.
    pub static ref WRITER: Mutex<Writer<Text80x25>> = Mutex::new(Writer {
        x_pos: 0,
        color: WriterColor::new(Color::White, Color::Black),
        size: BufferSize::new(80, 25),
        iw: Text80x25::new(),
    });
}

/// Represents buffer size.
///
/// Buffer size is *independent* from actual graphics settings. It should not be
/// bigger than actual graphics size.
pub struct BufferSize {
    x: usize,
    y: usize,
}

impl BufferSize {
    pub fn new(x: usize, y: usize) -> Self {
        BufferSize { x, y }
    }
}

/// A writer type that allows writing ASCII bytes and strings using.
///
/// Wraps lines at `size.x`. Supports newline characters and implements the
/// `core::fmt::Write` trait.
pub struct Writer<T: TextWriter> {
    x_pos: usize,
    color: WriterColor,
    size: BufferSize,
    iw: T,
}

impl<T: TextWriter> Writer<T> {
    /// Changes internal `TextWriter` and updates graphics mode.
    pub fn set_writer(&mut self, writer: T) {
        self.iw = writer;
        self.iw.set_mode();
    }

    /// Changes buffer size.
    pub fn set_size(&mut self, size: BufferSize) {
        self.size = size;
    }

    /// Writes an ASCII byte to the buffer.
    ///
    /// Wraps lines at `size.x`. Supports the `\n` newline character.
    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            0x08 => self.backspace(),
            byte => {
                if self.x_pos >= self.size.x {
                    self.new_line();
                }

                let screen_char = ScreenCharacter::new(byte, self.color);
                self.iw
                    .write_character(self.x_pos, self.size.y - 1, screen_char);

                self.x_pos += 1;
            }
        }
    }

    /// Writes the given ASCII string to the buffer.
    ///
    /// Wraps lines at `size.x`. Supports the `\n` newline character. Does **not**
    /// support strings with non-ASCII characters, since they can't be printed in the VGA text
    /// mode.
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // support for backspace
                0x08 => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
        // Update cursor position
        self.iw.set_cursor_position(self.x_pos, self.size.y - 1);
    }

    /// Shifts all lines one line up and clears the last line.
    pub fn new_line(&mut self) {
        for y in 1..self.size.y {
            for x in 0..self.size.x {
                let character = self.iw.read_character(x, y);
                self.iw.write_character(x, y - 1, character);
            }
        }
        self.clear_line(self.size.y - 1);
        self.x_pos = 0;
    }

    /// Shifts all lines one line down and clears the first line.
    pub fn del_line(&mut self) {
        for y in (0..self.size.y - 1).rev() {
            for x in 0..self.size.x {
                let character = self.iw.read_character(x, y);
                self.iw.write_character(x, y + 1, character);
            }
        }
        self.clear_line(0);
        self.x_pos = self.size.x;
    }

    /// Clears a line by overwriting it with a blank character.
    pub fn clear_line(&mut self, y: usize) {
        let blank = ScreenCharacter::new(b' ', self.color);
        for x in 0..self.size.x {
            self.iw.write_character(x, y, blank);
        }
    }

    /// Clears the screen by filling it with a blank character.
    pub fn clear_screen(&mut self) {
        for y in 0..self.size.y {
            self.clear_line(y);
        }
    }

    /// Overwrites the last written character with a blank character.
    pub fn backspace(&mut self) {
        let blank = ScreenCharacter::new(b' ', self.color);
        if self.x_pos == 0 {
            self.del_line();
        }
        self.x_pos -= 1;
        self.iw.write_character(self.x_pos, self.size.y - 1, blank);
    }
}

impl<T: TextWriter> fmt::Write for Writer<T> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

/// Like the `print!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

/// Like the `println!` macro in the standard library, but prints to the VGA text buffer.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

/// Prints the text colored in given `WriterColor`. Changes the colors to old colors when printing finishes.
pub fn print_colored(color: WriterColor, text: &str) {
    let mut writer = WRITER.lock();
    let old_color = writer.color;
    writer.color = color;
    x86_64::instructions::interrupts::without_interrupts(|| {
        writer.write_string(text);
    });
    writer.color = old_color;
}

/// Prints the given formatted string to the VGA text buffer through the global `WRITER` instance.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // 'without_interrupts' prevent a deadlock if we try to print to things at the same time
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

/// Changes the global `WRITER` instance `WriterColor` to the given `WriterColor`.
pub fn change_writer_color(color: WriterColor) {
    WRITER.lock().color = color;
}

/// Clears the screen using the global `WRITER` instance.
pub fn clear_screen() {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().clear_screen();
    });
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
    use x86_64::instructions::interrupts;

    serial_print!("test_println_output... ");

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.iw.read_character(i, writer.size.y - 2);
            assert_eq!(char::from(screen_char.get_character()), c);
        }
    });

    serial_println!("[ok]");
}
