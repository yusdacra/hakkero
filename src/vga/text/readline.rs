use super::WriterColor;
use alloc::vec::Vec;
use pc_keyboard::DecodedKey;
use vga::colors::Color16;
use vga::writers::{Screen, ScreenCharacter, Text80x25, TextWriter};

pub struct Readline<T: TextWriter> {
    bg_color: Color16,
    fg_color: Color16,
    buf: Vec<u8>,
    pos: usize,
    min_pos: usize,
    max_pos: usize,
    line: usize,
    iw: T,
}

#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
impl<T: TextWriter> Readline<T> {
    pub fn new(
        iw: T,
        min_pos: usize,
        max_pos: usize,
        line: usize,
        bg_color: Color16,
        fg_color: Color16,
    ) -> Self {
        assert!(max_pos - min_pos > 0);
        assert!(max_pos <= T::WIDTH);
        iw.set_mode();
        Self {
            bg_color,
            fg_color,
            buf: Vec::with_capacity(max_pos - min_pos),
            pos: 0,
            min_pos,
            max_pos,
            line,
            iw,
        }
    }

    pub fn handle_key(&mut self, key: DecodedKey) -> Option<Vec<u8>> {
        let mut res = None;
        match key {
            DecodedKey::Unicode(character) => {
                let ch = character as u8;
                if ch >= 0x20 && ch <= 0x7e {
                    if self.pos < self.max_pos {
                        self.buf.push(ch);
                        self.offset_pos(1);
                    }
                } else if ch == 8 {
                    if self.pos > self.min_pos {
                        self.buf.remove(self.pos - 1);
                        self.offset_pos(-1);
                    }
                } else if ch == b'\n' {
                    res = Some(Vec::with_capacity(self.buf.capacity()));
                    res.as_mut().unwrap().append(&mut self.buf);
                    self.pos = self.min_pos;
                } else {
                    self.buf.push(0xfe);
                    self.offset_pos(1);
                }
                self.write_buf();
            }
            DecodedKey::RawKey(raw) => {
                use pc_keyboard::KeyCode;
                match raw {
                    KeyCode::ArrowLeft => {
                        self.offset_pos(-1);
                    }
                    KeyCode::ArrowRight => {
                        self.offset_pos(1);
                    }
                    KeyCode::ArrowUp => {
                        self.pos = self.buf.len();
                    }
                    KeyCode::ArrowDown => {
                        self.pos = self.min_pos;
                    }
                    _ => (),
                }
            }
        }
        self.iw.set_cursor_position(self.pos, self.line);
        res
    }

    fn write_buf(&self) {
        for x in self.min_pos..self.max_pos {
            let character = if let Some(ch) = self.buf.get(x - self.min_pos) {
                *ch
            } else {
                b' '
            };
            self.iw.write_character(
                x,
                self.line,
                ScreenCharacter::new(character, WriterColor::new(self.fg_color, self.bg_color)),
            );
        }
    }

    fn offset_pos(&mut self, by: isize) {
        let mut new_pos = self.pos as isize + by;
        let min_pos = self.min_pos as isize;
        let max_pos = self.max_pos as isize;
        let buf_len = self.buf.len() as isize;
        if new_pos > max_pos {
            new_pos = max_pos;
        } else if new_pos > buf_len {
            new_pos = buf_len;
        } else if new_pos < min_pos {
            new_pos = min_pos;
        }
        self.pos = new_pos as usize;
    }
}

impl Default for Readline<Text80x25> {
    fn default() -> Self {
        Self::new(
            Text80x25::new(),
            0,
            Text80x25::WIDTH,
            Text80x25::HEIGHT - 1,
            Color16::Black,
            Color16::White,
        )
    }
}

// TESTS

#[cfg(test)]
use crate::{serial_print, serial_println};
#[cfg(test)]
use alloc::vec;

#[test_case]
fn test_one_key_unicode(sp: &mut crate::serial::SerialPort) {
    serial_print!(sp, "test_one_key_unicode... ");
    let mut rl = Readline::default();
    rl.handle_key(DecodedKey::Unicode('a'));
    assert_eq!(*rl.buf.first().unwrap(), b'a');
    serial_println!(sp, "[ok]");
}

#[test_case]
fn test_one_key_unicode_full_buf(sp: &mut crate::serial::SerialPort) {
    serial_print!(sp, "test_one_key_unicode_full_buf... ");
    let mut rl = Readline::default();
    rl.buf = vec![b'a'; 80];
    rl.pos = 80;
    rl.handle_key(DecodedKey::Unicode('b'));
    assert_eq!(*rl.buf.last().unwrap(), b'a');
    serial_println!(sp, "[ok]");
}

#[test_case]
fn test_left_empty_buf(sp: &mut crate::serial::SerialPort) {
    serial_print!(sp, "test_left_empty_buf... ");
    let mut rl = Readline::default();
    rl.handle_key(DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowLeft));
    assert_eq!(rl.pos, 0);
    serial_println!(sp, "[ok]");
}

#[test_case]
fn test_right_empty_buf(sp: &mut crate::serial::SerialPort) {
    serial_print!(sp, "test_right_empty_buf... ");
    let mut rl = Readline::default();
    rl.handle_key(DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowRight));
    assert_eq!(rl.pos, 0);
    serial_println!(sp, "[ok]");
}

#[test_case]
fn test_up_empty_buf(sp: &mut crate::serial::SerialPort) {
    serial_print!(sp, "test_up_empty_buf... ");
    let mut rl = Readline::default();
    rl.handle_key(DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowUp));
    assert_eq!(rl.pos, 0);
    serial_println!(sp, "[ok]");
}

#[test_case]
fn test_down_empty_buf(sp: &mut crate::serial::SerialPort) {
    serial_print!(sp, "test_down_empty_buf... ");
    let mut rl = Readline::default();
    rl.handle_key(DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowDown));
    assert_eq!(rl.pos, 0);
    serial_println!(sp, "[ok]");
}

#[test_case]
fn test_left(sp: &mut crate::serial::SerialPort) {
    serial_print!(sp, "test_left... ");
    let mut rl = Readline::default();
    rl.buf = vec![b'a'];
    rl.pos = 1;
    rl.handle_key(DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowLeft));
    assert_eq!(rl.pos, 0);
    serial_println!(sp, "[ok]");
}

#[test_case]
fn test_right(sp: &mut crate::serial::SerialPort) {
    serial_print!(sp, "test_right... ");
    let mut rl = Readline::default();
    rl.buf = vec![b'a'];
    rl.handle_key(DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowRight));
    assert_eq!(rl.pos, 1);
    serial_println!(sp, "[ok]");
}

#[test_case]
fn test_right_full_buf(sp: &mut crate::serial::SerialPort) {
    serial_print!(sp, "test_right_full_buf... ");
    let mut rl = Readline::default();
    rl.buf = vec![b'a'; 80];
    rl.pos = 80;
    rl.handle_key(DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowRight));
    assert_eq!(rl.pos, 80);
    serial_println!(sp, "[ok]");
}

#[test_case]
fn test_up(sp: &mut crate::serial::SerialPort) {
    serial_print!(sp, "test_up... ");
    let mut rl = Readline::default();
    rl.buf = vec![b'a'];
    rl.handle_key(DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowUp));
    assert_eq!(rl.pos, 1);
    serial_println!(sp, "[ok]");
}

#[test_case]
fn test_down(sp: &mut crate::serial::SerialPort) {
    serial_print!(sp, "test_down_empty_buf... ");
    let mut rl = Readline::default();
    rl.buf = vec![b'a'];
    rl.pos = 1;
    rl.handle_key(DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowDown));
    assert_eq!(rl.pos, 0);
    serial_println!(sp, "[ok]");
}

#[test_case]
fn test_newline(sp: &mut crate::serial::SerialPort) {
    serial_print!(sp, "test_newline... ");
    let mut rl = Readline::default();
    rl.buf = vec![b'a'];
    let res = rl.handle_key(DecodedKey::Unicode('\n')).unwrap();
    assert_eq!(*res.first().unwrap(), b'a');
    serial_println!(sp, "[ok]");
}

#[test_case]
fn test_newline_empty_buf(sp: &mut crate::serial::SerialPort) {
    serial_print!(sp, "test_newline_empty_buf... ");
    let mut rl = Readline::default();
    let res = rl.handle_key(DecodedKey::Unicode('\n')).unwrap();
    assert!(res.is_empty());
    serial_println!(sp, "[ok]");
}

#[test_case]
fn test_backspace(sp: &mut crate::serial::SerialPort) {
    serial_print!(sp, "test_backspace... ");
    let mut rl = Readline::default();
    rl.buf = vec![b'a'];
    rl.pos = 1;
    rl.handle_key(DecodedKey::Unicode('\u{8}'));
    assert!(rl.buf.is_empty());
    assert_eq!(rl.pos, 0);
    serial_println!(sp, "[ok]");
}

#[test_case]
fn test_backspace_empty_buf(sp: &mut crate::serial::SerialPort) {
    serial_print!(sp, "test_backspace_empty_buf... ");
    let mut rl = Readline::default();
    rl.handle_key(DecodedKey::Unicode('\u{8}'));
    assert_eq!(rl.pos, 0);
    serial_println!(sp, "[ok]");
}
