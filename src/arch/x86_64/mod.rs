pub mod device;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod task;

use bootloader::BootInfo;

#[allow(clippy::inline_always)]
#[inline(always)] // Inline because it will be only used once anyways
pub fn start(boot_info: &'static BootInfo) {
    crate::common::init();
    gdt::init();
    interrupts::init_idt();
    device::init();
    init_heap(boot_info);
}

/// Initializes the heap.
/// This gets the mapper and a `BootInfoFrameAllocator` from the given `BootInfo`, then calls `setup_heap` from the `allocator` module.
#[allow(clippy::inline_always)]
#[inline(always)] // Inline because it will be only used once anyways
pub fn init_heap(boot_info: &'static BootInfo) {
    let phys_mem_offset = x86_64::VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };

    memory::setup_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub use x86_64::instructions::interrupts::without_interrupts as woint;
