use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

#[cfg(test)]
use crate::{serial_print, serial_println};

lazy_static! {
    /// A global `Writer` instance that can be used for printing to the VGA text buffer.
    ///
    /// Used by the `print!` and `println!` macros.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        cursor: {
            let mut cur = Cursor::new(1);
            cur.set_state(true);
            cur
        },
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

/// The standard color palette in VGA text mode.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// A combination of a foreground and a background color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    /// Create a new `ColorCode` with the given foreground and background colors.
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

use x86_64::instructions::port::Port;

/// A structure for holding information about the VGA text cursor.
pub struct Cursor {
    cur_port: Port<u8>,
    cur_port_pos: Port<u8>,
    cursor_size: u8, // at max 14, otherwise panic
}

impl Cursor {
    fn new(cursor_size: u8) -> Cursor {
        assert!(cursor_size < 15);

        Cursor {
            cur_port: Port::new(0x3D4),
            cur_port_pos: Port::new(0x3D5),
            cursor_size,
        }
    }

    /// Enable or disable cursor.
    fn set_state(&mut self, state: bool) {
        if state {
            unsafe {
                self.cur_port.write(0x0A);
                let mut som = self.cur_port_pos.read() & 0xC0 | 1;
                self.cur_port_pos.write(som);
                self.cur_port_pos.write(0x0B);
                som = self.cur_port_pos.read() & 0xE0 | 15 - self.cursor_size;
                self.cur_port_pos.write(som);
            }
        } else {
            unsafe {
                self.cur_port.write(0x0A);
                self.cur_port_pos.write(0x20);
            }
        }
    }

    /// Set cursor position, x equals to column position and y equals to row position.
    fn set_position(&mut self, x: u16, y: u16) {
        let pos: u16 = y * BUFFER_WIDTH as u16 + x;

        unsafe {
            self.cur_port.write(0x0F);
            self.cur_port_pos.write((pos & 0xFF) as u8);
            self.cur_port.write(0x0E);
            self.cur_port_pos.write(((pos >> 8) & 0xFF) as u8);
        }
    }
}

/// A screen character in the VGA text buffer, consisting of an ASCII character and a `ColorCode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

/// The height of the text buffer (normally 25 lines).
const BUFFER_HEIGHT: usize = 25;
/// The width of the text buffer (normally 80 columns).
const BUFFER_WIDTH: usize = 80;

/// A structure representing the VGA text buffer.
#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

/// A writer type that allows writing ASCII bytes and strings to an underlying `Buffer`.
///
/// Wraps lines at `BUFFER_WIDTH`. Supports newline characters and implements the
/// `core::fmt::Write` trait.
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    cursor: Cursor,
    buffer: &'static mut Buffer,
}

impl Writer {
    /// Writes an ASCII byte to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            0x08 => self.backspace(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    /// Writes the given ASCII string to the buffer.
    ///
    /// Wraps lines at `BUFFER_WIDTH`. Supports the `\n` newline character. Does **not**
    /// support strings with non-ASCII characters, since they can't be printed in the VGA text
    /// mode.
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // support for backspace
                0x08 => self.write_byte(0x08),
                // not part of printable ASCII range
                _ => self.write_byte(0xfe),
            }
        }
        // Update cursor position
        self.cursor
            .set_position(self.column_position as u16, BUFFER_HEIGHT as u16 - 1);
    }

    /// Shifts all lines one line up and clears the last row.
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    /// Shifts all lines one line down and clears the first row.
    fn del_line(&mut self) {
        for row in (0..BUFFER_HEIGHT - 1).rev() {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row + 1][col].write(character);
            }
        }
        self.clear_row(0);
        self.column_position = BUFFER_WIDTH;
    }

    /// Clears a row by overwriting it with blank characters.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    /// Overwrites the last written character with a blank character
    fn backspace(&mut self) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        if self.column_position == 0 {
            self.del_line();
        }
        self.column_position -= 1;
        self.buffer.chars[BUFFER_HEIGHT - 1][self.column_position].write(blank);
    }
}

impl fmt::Write for Writer {
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

/// Changes the global `WRITER` instance `ColorCode` to the given colors.
pub fn change_writer_color(foreground: Color, background: Color) {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().color_code = ColorCode::new(foreground, background);
    });
}

pub fn clear_row(row: usize) {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().clear_row(row);
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
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });

    serial_println!("[ok]");
}
