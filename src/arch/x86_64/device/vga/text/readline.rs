use pc_keyboard::DecodedKey;
use smallstr::SmallString;

const WIDTH: usize = 80;

pub struct Readline {
    buf: SmallString<[u8; WIDTH]>,
    pos: usize,
}

#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
impl Readline {
    pub fn new() -> Self {
        Self {
            buf: SmallString::with_capacity(WIDTH),
            pos: 0,
        }
    }

    /// Takes a `DecodedKey`. Returns the content of inner buffer when `'\n'` is pressed.
    pub fn handle_key(&mut self, key: DecodedKey) -> Option<SmallString<[u8; WIDTH]>> {
        match key {
            DecodedKey::Unicode(character) => {
                match character as u8 {
                    0x20..=0x7e => {
                        self.buf.insert(self.pos, character);
                        self.offset_pos(1);
                    }
                    0x08 if self.pos > 0 => {
                        self.offset_pos(-1);
                        self.buf.remove(self.pos);
                    }
                    0x7f if self.pos < self.buf.len() => {
                        self.buf.remove(self.pos);
                    }
                    b'\n' => {
                        let mut res = SmallString::with_capacity(self.buf.capacity());
                        for c in self.buf.drain() {
                            res.push(c);
                        }
                        self.buf.shrink_to_fit();
                        self.pos = 0;
                        self.write_buf();
                        self.iw.set_cursor_position(self.pos, HEIGHT - 1);
                        return Some(res);
                    }
                    _ => (),
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
                        self.pos = 0;
                    }
                    _ => (),
                }
            }
        }
        self.iw.set_cursor_position(self.pos, HEIGHT - 1);
        None
    }

    fn write_buf(&self) {
        let mut bytes = self.buf.bytes();
        for x in 0..WIDTH {
            let is_end = self.buf.len() > WIDTH && x == WIDTH - 1;
            let character = if is_end {
                b'>'
            } else if let Some(b) = bytes.next() {
                b
            } else {
                b' '
            };
            let color = TextModeColor::new(
                if is_end { BG_COLOR } else { FG_COLOR },
                if is_end { FG_COLOR } else { BG_COLOR },
            );
            self.iw
                .write_character(x, HEIGHT - 1, ScreenCharacter::new(character, color));
        }
    }

    fn offset_pos(&mut self, by: isize) {
        let mut new_pos = self.pos as isize + by;
        let buf_len = self.buf.len() as isize;
        if new_pos > buf_len {
            new_pos = buf_len;
        } else if new_pos < 0 {
            new_pos = 0;
        }
        self.pos = new_pos as usize;
    }
}

// TESTS

#[cfg(test)]
use {
    crate::{serial_print, serial_println},
    alloc::vec,
};

#[test_case]
fn test_one_key_unicode() {
    serial_print!("test_one_key_unicode... ");
    let mut rl = Readline::new();
    rl.handle_key(DecodedKey::Unicode('a'));
    assert_eq!(&rl.buf, "a");
    assert_eq!(rl.pos, 1);
    serial_println!("[ok]");
}

#[test_case]
fn test_one_key_insert_unicode() {
    serial_print!("test_one_key_insert_unicode... ");
    let mut rl = Readline::new();
    rl.buf = SmallString::from("aa");
    rl.pos = 1;
    rl.handle_key(DecodedKey::Unicode('b'));
    assert_eq!(rl.pos, 2);
    assert_eq!(&rl.buf, "aba");
    serial_println!("[ok]");
}

#[test_case]
fn test_left_empty_buf() {
    serial_print!("test_left_empty_buf... ");
    let mut rl = Readline::new();
    rl.handle_key(DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowLeft));
    assert_eq!(rl.pos, 0);
    serial_println!("[ok]");
}

#[test_case]
fn test_right_empty_buf() {
    serial_print!("test_right_empty_buf... ");
    let mut rl = Readline::new();
    rl.handle_key(DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowRight));
    assert_eq!(rl.pos, 0);
    serial_println!("[ok]");
}

#[test_case]
fn test_up_empty_buf() {
    serial_print!("test_up_empty_buf... ");
    let mut rl = Readline::new();
    rl.handle_key(DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowUp));
    assert_eq!(rl.pos, 0);
    serial_println!("[ok]");
}

#[test_case]
fn test_down_empty_buf() {
    serial_print!("test_down_empty_buf... ");
    let mut rl = Readline::new();
    rl.handle_key(DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowDown));
    assert_eq!(rl.pos, 0);
    serial_println!("[ok]");
}

#[test_case]
fn test_left() {
    serial_print!("test_left... ");
    let mut rl = Readline::new();
    rl.buf = SmallString::from("a");
    rl.pos = 1;
    rl.handle_key(DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowLeft));
    assert_eq!(rl.pos, 0);
    serial_println!("[ok]");
}

#[test_case]
fn test_right() {
    serial_print!("test_right... ");
    let mut rl = Readline::new();
    rl.buf = SmallString::from("a");
    rl.handle_key(DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowRight));
    assert_eq!(rl.pos, 1);
    serial_println!("[ok]");
}

#[test_case]
fn test_right_full_buf() {
    serial_print!("test_right_full_buf... ");
    let mut rl = Readline::new();
    rl.buf = vec![b'a'; 80].into_iter().map(|b| b as char).collect();
    rl.pos = 80;
    rl.handle_key(DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowRight));
    assert_eq!(rl.pos, 80);
    serial_println!("[ok]");
}

#[test_case]
fn test_up() {
    serial_print!("test_up... ");
    let mut rl = Readline::new();
    rl.buf = SmallString::from("a");
    rl.handle_key(DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowUp));
    assert_eq!(rl.pos, 1);
    serial_println!("[ok]");
}

#[test_case]
fn test_down() {
    serial_print!("test_down_empty_buf... ");
    let mut rl = Readline::new();
    rl.buf = SmallString::from("a");
    rl.pos = 1;
    rl.handle_key(DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowDown));
    assert_eq!(rl.pos, 0);
    serial_println!("[ok]");
}

#[test_case]
fn test_newline() {
    serial_print!("test_newline... ");
    let mut rl = Readline::new();
    rl.buf = SmallString::from("a");
    let res = rl.handle_key(DecodedKey::Unicode('\n')).unwrap();
    assert_eq!(&res, "a");
    serial_println!("[ok]");
}

#[test_case]
fn test_newline_empty_buf() {
    serial_print!("test_newline_empty_buf... ");
    let mut rl = Readline::new();
    let res = rl.handle_key(DecodedKey::Unicode('\n')).unwrap();
    assert!(res.is_empty());
    serial_println!("[ok]");
}

#[test_case]
fn test_backspace() {
    serial_print!("test_backspace... ");
    let mut rl = Readline::new();
    rl.buf = SmallString::from("a");
    rl.pos = 1;
    rl.handle_key(DecodedKey::Unicode('\u{8}'));
    assert!(rl.buf.is_empty());
    assert_eq!(rl.pos, 0);
    serial_println!("[ok]");
}

#[test_case]
fn test_backspace_empty_buf() {
    serial_print!("test_backspace_empty_buf... ");
    let mut rl = Readline::new();
    rl.handle_key(DecodedKey::Unicode('\u{8}'));
    assert_eq!(rl.pos, 0);
    serial_println!("[ok]");
}

#[test_case]
fn test_delete() {
    serial_print!("test_delete... ");
    let mut rl = Readline::new();
    rl.buf = SmallString::from("a");
    rl.handle_key(DecodedKey::Unicode('\u{7f}'));
    assert!(rl.buf.is_empty());
    assert_eq!(rl.pos, 0);
    serial_println!("[ok]");
}

#[test_case]
fn test_delete_empty_buf() {
    serial_print!("test_delete_empty_buf... ");
    let mut rl = Readline::new();
    rl.handle_key(DecodedKey::Unicode('\u{7f}'));
    assert_eq!(rl.pos, 0);
    serial_println!("[ok]");
}
