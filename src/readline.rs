use crate::print;
use alloc::string::String;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    /// The global `Readline`. Used by the kernel shell.
    pub static ref RL: Mutex<Readline> = Mutex::new(Readline::new());
}

/// Handles characters and stores them inside a buffer.
pub struct Readline {
    pos: usize,
    buf: String,
}

impl Readline {
    pub fn new() -> Readline {
        Readline {
            pos: 0,
            buf: String::new(),
        }
    }

    pub fn handle_character(&mut self, character: char) {
        if character == '\n' {
            // Readline doesn't handle newlines. Name checks out.
            return;
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
        if self.buf.is_empty() {
            None
        } else {
            Some({
                let res = self.buf.clone();
                self.buf.clear();
                res
            })
        }
    }
}

/// Handles a character by passing it to the global `Readline`.
pub fn handle_character(char: char) {
    RL.lock().handle_character(char);
}

/// Retrieves the data currently stored in the global `Readline`.
pub fn retrieve_data() -> Option<String> {
    RL.lock().retrieve_data()
}
