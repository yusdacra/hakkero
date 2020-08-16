pub mod pic8259;
pub mod uart16550;
pub mod vga;

pub fn init() {
    uart16550::init();
    vga::text::init();
    pic8259::init();
}
