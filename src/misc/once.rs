//! Structure that allows one time initialization of memory.
use core::cell::UnsafeCell;
use core::mem::MaybeUninit;
use core::sync::atomic::{AtomicBool, Ordering};

/// Allows one time initialization.
pub struct Once<T> {
    state: AtomicBool, // `true` means initialized.
    data: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Send + Sync> Sync for Once<T> {}
unsafe impl<T: Send> Send for Once<T> {}

impl<T> Once<T> {
    /// Create a new, uninitialized `Once`.
    pub const fn new() -> Self {
        Self {
            state: AtomicBool::new(false),
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Try to initialize `Once` with given data.
    ///
    /// Returns a reference to the data held inside.
    /// If `Once` is already initialized this function will do nothing.
    pub fn try_init(&self, data: T) -> &T {
        if self.state.compare_and_swap(false, true, Ordering::AcqRel) {
            unsafe { self.get_unchecked() }
        } else {
            unsafe {
                (*self.data.get()).as_mut_ptr().write(data);
                self.get_unchecked()
            }
        }
    }

    /// Get a reference to the data held inside, without performing safety checks.
    ///
    /// # Safety
    /// You must make sure that `Once` is initialized before calling this function.
    pub unsafe fn get_unchecked(&self) -> &T {
        (*self.data.get()).get_ref()
    }

    /// Get a reference to the data held inside.
    ///
    /// Returns `None` if `Once` wasn't initialized, `Some(&T)` otherwise.
    pub fn get(&self) -> Option<&T> {
        if !self.state.load(Ordering::Relaxed) {
            return None;
        }
        Some(unsafe { self.get_unchecked() })
    }
}

#[cfg(test)]
use crate::{serial_print, serial_println};

#[test_case]
fn test_once_notinit() {
    serial_print!("test_once_notinit... ");
    let once: Once<bool> = Once::new();
    assert_eq!(once.get(), None);
    serial_println!("[ok]");
}

#[test_case]
fn test_once_init() {
    serial_print!("test_once_init... ");
    let once: Once<bool> = Once::new();
    assert_eq!(*once.try_init(true), true);
    assert_eq!(*once.get().unwrap(), true);
    serial_println!("[ok]");
}
