pub mod readline;

use crate::arch::x86_64::woint;
use core::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use vga::{
    colors::{Color16, TextModeColor},
    writers::{Screen, ScreenCharacter, Text80x25, TextWriter},
};

static mut WRITER: Writer = Writer::new();

const HEIGHT: usize = Text80x25::HEIGHT;
const WIDTH: usize = Text80x25::WIDTH;
const BG_COLOR: Color16 = Color16::Black;
const FG_COLOR: Color16 = Color16::White;

struct WriterColor {
    colors: AtomicU8,
}

impl WriterColor {
    fn set_colors(&self, colors: (Color16, Color16)) {
        self.colors.store(Self::encode(colors), Ordering::Relaxed);
    }

    fn get_colors(&self) -> (Color16, Color16) {
        Self::decode(self.colors.load(Ordering::Relaxed)).unwrap()
    }

    fn get_text_color(&self) -> TextModeColor {
        let colors = Self::decode(self.colors.load(Ordering::Relaxed)).unwrap();
        TextModeColor::new(colors.0, colors.1)
    }

    fn u8_to_color16(n: u8) -> Option<Color16> {
        Some(match n {
            0 => Color16::Black,
            1 => Color16::Blue,
            2 => Color16::Green,
            3 => Color16::Cyan,
            4 => Color16::Red,
            5 => Color16::Magenta,
            6 => Color16::Brown,
            7 => Color16::LightGrey,
            8 => Color16::DarkGrey,
            9 => Color16::LightBlue,
            10 => Color16::LightGreen,
            11 => Color16::LightCyan,
            12 => Color16::LightRed,
            13 => Color16::Pink,
            14 => Color16::Yellow,
            15 => Color16::White,
            _ => return None,
        })
    }

    fn encode(colors: (Color16, Color16)) -> u8 {
        u8::from(colors.0) << 4 | u8::from(colors.1)
    }

    fn decode(colors: u8) -> Option<(Color16, Color16)> {
        let fg = Self::u8_to_color16(colors >> 4)?;
        let bg = Self::u8_to_color16(colors & 0x0f)?;
        Some((fg, bg))
    }
}

/// A writer type that allows writing ASCII bytes and strings using.
///
/// Wraps lines at `size.x`. Supports newline characters and implements the
/// `core::fmt::Write` trait.
pub struct Writer {
    color: WriterColor,
    x_pos: AtomicUsize,
    iw: Text80x25,
}

impl Writer {
    /// Creates a new `Writer`.
    pub const fn new() -> Self {
        Self {
            color: WriterColor {
                colors: AtomicU8::new(0xF << 4),
            },
            x_pos: AtomicUsize::new(0),
            iw: Text80x25::new(),
        }
    }

    pub fn init(&self) {
        self.iw.set_mode();
    }

    /// Writes an ASCII byte to the buffer at current position.
    fn write_byte(&self, byte: u8) {
        self.iw.write_character(
            self.x_pos.load(Ordering::Relaxed),
            HEIGHT - 1,
            ScreenCharacter::new(byte, self.color.get_text_color()),
        );
    }

    /// Writes the given ASCII string to the buffer.
    ///
    /// Wraps lines at `size.x`. Supports the `\n` newline character. Does **not**
    /// support strings with non-ASCII characters, since they can't be printed in the VGA text
    /// mode.
    fn write_string(&self, s: &str) {
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
            if self.x_pos.fetch_add(1, Ordering::Relaxed) >= WIDTH {
                self.new_line();
            }
        }
        // Update cursor position
        self.iw
            .set_cursor_position(self.x_pos.load(Ordering::Relaxed), HEIGHT - 1);
    }

    /// Clears a line by overwriting it with a blank character.
    fn clear_line(&self, y: usize) {
        for x in 0..WIDTH {
            self.iw.write_character(
                x,
                y,
                ScreenCharacter::new(b' ', TextModeColor::new(FG_COLOR, BG_COLOR)),
            );
        }
    }

    /// Clears the last line and shifts all lines one line upwards.
    fn new_line(&self) {
        for y in 1..HEIGHT {
            for x in 0..WIDTH {
                let character = self.iw.read_character(x, y);
                self.iw.write_character(x, y - 1, character);
            }
        }
        self.clear_line(HEIGHT - 1);
        self.x_pos.store(0, Ordering::Relaxed);
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::arch::device::vga::text::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print_colored {
    ($color:expr, $($arg:tt)*) => ($crate::arch::device::vga::text::_print_colored($color, format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println_colored {
    ($color:expr) => ($crate::print_colored!($color, "\n"));
    ($color:expr, $($arg:tt)*) => ($crate::print_colored!($color, "{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print_colored(colors: (Color16, Color16), args: core::fmt::Arguments) {
    use core::fmt::Write;

    woint(|| {
        let old_colors = unsafe { &WRITER }.color.get_colors();
        unsafe { &WRITER }.color.set_colors(colors);
        // This isn't actually "unsafe" because we dont mutate anything in `write_string` function
        write!(unsafe { &mut WRITER }, "{}", args)
            .expect("failed to write to vga buffer (literally how)");
        unsafe { &WRITER }.color.set_colors(old_colors);
    });
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;

    woint(|| {
        write!(unsafe { &mut WRITER }, "{}", args)
            .expect("failed to write to vga buffer (literally how)")
    });
}

// TESTS

#[cfg(test)]
use crate::{serial_print, serial_println};

#[test_case]
fn test_println_output() {
    serial_print!("test_println_output... ");

    let s = "1234567890";
    println!("\n{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = unsafe { &WRITER }.iw.read_character(i, HEIGHT - 2);
        assert_eq!(screen_char.get_character(), c as u8);
    }

    serial_println!("[ok]");
}

#[test_case]
fn test_println_colored_output() {
    serial_print!("test_println_colored_output... ");

    let s = "1234567890";
    let colors = (Color16::Black, Color16::Brown);
    println_colored!(colors, "\n{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = unsafe { &WRITER }.iw.read_character(i, HEIGHT - 2);
        assert_eq!(screen_char.get_character(), c as u8);
        assert_eq!(
            screen_char.get_color(),
            TextModeColor::new(colors.0, colors.1)
        );
    }

    serial_println!("[ok]");
}

#[test_case]
fn test_println_fit_line_output() {
    serial_print!("test_println_fit_line_output... ");

    for _ in 0..WIDTH {
        print!("a");
    }
    assert_eq!(
        char::from(
            unsafe { &WRITER }
                .iw
                .read_character(unsafe { &WRITER }.x_pos.load(Ordering::Relaxed), HEIGHT - 1)
                .get_character()
        ),
        ' '
    );

    serial_println!("[ok]");
}
