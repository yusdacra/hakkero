pub mod pic8259;
pub mod uart16550;
pub mod vga;

pub fn init() {
    pic8259::init();
    uart16550::init();
}
