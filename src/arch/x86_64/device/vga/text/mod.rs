pub mod readline;

use crate::arch::x86_64::woint;
use spinning_top::Spinlock;
use vga::{
    colors::{Color16, TextModeColor},
    writers::{Screen, ScreenCharacter, Text80x25, TextWriter},
};

static WRITER: Spinlock<Writer> = Spinlock::new(Writer::new());

/// Initializes the `WRITER`.
pub fn init() {
    log::trace!("Initializing VGA writer...");
    WRITER.lock().init();
    log::info!("Successfully initialized VGA writer!");
}

const HEIGHT: usize = Text80x25::HEIGHT;
const WIDTH: usize = Text80x25::WIDTH;
const BG_COLOR: Color16 = Color16::Black;
const FG_COLOR: Color16 = Color16::White;

struct WriterColor {
    colors: u8,
}

impl WriterColor {
    fn set_colors(&mut self, colors: (Color16, Color16)) {
        self.colors = Self::encode(colors);
    }

    fn get_colors(&self) -> (Color16, Color16) {
        Self::decode(self.colors).unwrap()
    }

    fn get_text_color(&self) -> TextModeColor {
        let colors = Self::decode(self.colors).unwrap();
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
/// Wraps lines at ``. Supports newline characters and implements the
/// `core::fmt::Write` trait.
pub struct Writer {
    color: WriterColor,
    x_pos: usize,
    iw: Text80x25,
    is_init: bool,
}

impl Writer {
    /// Creates a new `Writer`.
    pub const fn new() -> Self {
        Self {
            color: WriterColor { colors: 0xF << 4 },
            x_pos: 0,
            iw: Text80x25::new(),
            is_init: false,
        }
    }

    pub fn init(&mut self) {
        self.iw.set_mode();
        self.iw
            .fill_screen(ScreenCharacter::new(b' ', self.color.get_text_color()));
        self.is_init = true;
    }

    /// Writes an ASCII byte to the buffer at current position.
    fn write_byte(&mut self, byte: u8) {
        if self.x_pos >= WIDTH {
            self.new_line();
        }
        let wb = match byte {
            // printable ASCII byte
            0x20..=0x7e => byte,
            // newline
            b'\n' => {
                self.new_line();
                return; // here to skip x offset
            }
            // not part of printable ASCII range
            _ => 0xfe,
        };
        self.iw.write_character(
            self.x_pos,
            HEIGHT - 1,
            ScreenCharacter::new(wb, self.color.get_text_color()),
        );
        self.x_pos += 1;
    }

    /// Writes the given ASCII string to the buffer.
    ///
    /// Wraps lines at `size.x`. Supports the `\n` newline character. Does **not**
    /// support strings with non-ASCII characters, since they can't be printed in the VGA text
    /// mode.
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        // Update cursor position
        self.iw.set_cursor_position(self.x_pos, HEIGHT - 1);
    }

    /// Clears a line by overwriting it with a blank character.
    fn clear_line(&mut self, y: usize) {
        for x in 0..WIDTH {
            self.iw.write_character(
                x,
                y,
                ScreenCharacter::new(b' ', TextModeColor::new(FG_COLOR, BG_COLOR)),
            );
        }
    }

    /// Clears the last line and shifts all lines one line upwards.
    fn new_line(&mut self) {
        for y in 1..HEIGHT {
            for x in 0..WIDTH {
                let character = self.iw.read_character(x, y);
                self.iw.write_character(x, y - 1, character);
            }
        }
        self.clear_line(HEIGHT - 1);
        self.x_pos = 0;
    }
}

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

pub macro print($($arg:tt)*) {
    $crate::arch::device::vga::text::_print(format_args!($($arg)*));
}

pub macro println($($arg:tt)*) {
    $crate::print!("{}\n", format_args!($($arg)*));
}

pub macro print_colored($color:expr, $($arg:tt)*) {
    $crate::arch::device::vga::text::_print_colored($color, format_args!($($arg)*));
}

pub macro println_colored($color:expr, $($arg:tt)*) {
    $crate::print_colored!($color, "{}\n", format_args!($($arg)*));
}

#[doc(hidden)]
pub fn _print_colored(colors: (Color16, Color16), args: core::fmt::Arguments) {
    use core::fmt::Write;

    let mut writer = WRITER.lock();
    if !writer.is_init {
        return;
    }

    woint(|| {
        let old_colors = writer.color.get_colors();
        writer.color.set_colors(colors);
        write!(&mut writer, "{}", args).expect("failed to write to vga buffer (literally how)");
        writer.color.set_colors(old_colors);
    });
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    use core::fmt::Write;

    let mut writer = WRITER.lock();
    if !writer.is_init {
        return;
    }

    woint(|| {
        write!(&mut writer, "{}", args).expect("failed to write to vga buffer (literally how)")
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
    let writer = WRITER.lock();
    for (i, c) in s.chars().enumerate() {
        let screen_char = writer.iw.read_character(i, HEIGHT - 2);
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
    let writer = WRITER.lock();
    for (i, c) in s.chars().enumerate() {
        let screen_char = writer.iw.read_character(i, HEIGHT - 2);
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

    let writer = WRITER.lock();
    assert_eq!(
        char::from(
            writer
                .iw
                .read_character(writer.x_pos, HEIGHT - 1)
                .get_character()
        ),
        ' '
    );

    serial_println!("[ok]");
}
