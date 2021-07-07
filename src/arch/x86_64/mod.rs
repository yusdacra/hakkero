//! `x86_64` specific code.
pub mod device;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod task;

use bootloader::{boot_info::MemoryRegions, BootInfo};

/// Initializes the GDT, interrupts, devices and lastly the heap.
///
/// # Safety
/// Must only be called once.
#[allow(clippy::inline_always)]
#[inline(always)] // Inline because it will be only used once anyways
pub unsafe fn init(boot_info: &'static mut BootInfo) {
    crate::logger::init();
    gdt::init();
    interrupts::init_idt();
    device::init(boot_info.framebuffer.as_mut());
    init_heap(
        boot_info.physical_memory_offset.into_option(),
        &boot_info.memory_regions,
    );
    log::info!("Initialized all peripherals!");
}

/// Initializes the heap.
/// This gets the mapper and a `BootInfoFrameAllocator` from the given `BootInfo`, then calls `setup_heap` from the `memory` module.
///
/// # Safety
/// Must only be called once.
///
/// # Panics
/// Panics if the heap can't be initialized.
#[allow(clippy::inline_always)]
#[inline(always)] // Inline because it will be only used once anyways
pub unsafe fn init_heap(
    physical_memory_offset: Option<u64>,
    memory_regions: &'static MemoryRegions,
) {
    let phys_mem_offset = x86_64::VirtAddr::new(physical_memory_offset.unwrap());
    let mut mapper = memory::init_offset_page_table(phys_mem_offset);
    let mut frame_allocator = memory::BootInfoFrameAllocator::init(memory_regions);

    memory::setup_heap(&mut mapper, &mut frame_allocator).expect("Heap initialization failed");
}

/// Make an entry point. This macro checks the signature of the provided
/// function to make sure it's correct.
pub macro entry_point($path:path) {
    bootloader::entry_point!(kernel_init);
    fn kernel_init(boot_info: &'static mut bootloader::BootInfo) -> ! {
        let entry: fn() -> ! = $path;
        unsafe {
            $crate::arch::init(boot_info);
        }
        entry()
    }
}

#[allow(clippy::inline_always)]
#[inline(always)]
pub fn hang_cpu() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

pub use x86_64::instructions::interrupts::without_interrupts as woint;
