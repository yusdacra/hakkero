use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use spinning_top::{Spinlock, SpinlockGuard};

pub mod bump;
pub mod fixed_size_block;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 1000 * 1024; // 1000 KiB

#[global_allocator]
pub static ALLOCATOR: Locked<fixed_size_block::Allocator> =
    Locked::new(fixed_size_block::Allocator::new());

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

/// Just a dummy allocator which does nothing.
pub struct Dummy;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called")
    }
}

pub struct Locked<A> {
    inner: Spinlock<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: Spinlock::new(inner),
        }
    }

    pub fn lock(&self) -> SpinlockGuard<A> {
        self.inner.lock()
    }
}

/// Align the given address `addr` upwards to alignment `align`.
///
/// # Safety
/// `align` must be a power of two.
unsafe fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
