use crate::alloc::borrow::ToOwned;
use crate::print;
use alloc::string::String;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref RL: Mutex<Readline> = Mutex::new(Readline::new());
}

pub struct Readline {
    pos: usize,
    buf: String,
    retrieve: bool,
}

impl Readline {
    pub fn new() -> Readline {
        Readline {
            pos: 0,
            buf: String::new(),
            retrieve: false,
        }
    }

    pub fn handle_character(&mut self, character: char) {
        if character == '\n' {
            print!("{}", character);
            self.retrieve = true;
        } else if self.pos > 0 && character == '\u{8}' {
            print!("{}", character);
            self.pos -= 1;
            self.buf.remove(self.pos);
        } else if character != '\u{8}' {
            print!("{}", character);
            self.pos += 1;
            self.buf.push(character);
        }
    }

    pub fn retrieve_data(&mut self) -> Option<String> {
        if self.retrieve {
            return Some({
                self.retrieve = false;
                self.pos = 0;
                let mut res = String::with_capacity(self.buf.capacity());
                self.buf.clone_into(&mut res);
                self.buf.clear();
                res
            });
        }

        None
    }
}
