use bootloader::boot_info::FrameBuffer;

pub mod pic8259;
pub mod uart16550;
pub mod vga;

pub fn init(framebuffer: Option<&'static mut FrameBuffer>) {
    pic8259::init();
    uart16550::init();
    if let Some(framebuffer) = framebuffer {
        vga::text::WRITER.call_once(move || {
            let info = framebuffer.info();
            let buf = framebuffer.buffer_mut();
            vga::text::LockedWriter::new(buf, info)
        });
    }
}
