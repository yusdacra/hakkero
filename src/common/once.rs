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
        if self.state.load(Ordering::SeqCst) {
            return unsafe { self.get_unchecked() };
        }
        unsafe {
            (*self.data.get()).as_mut_ptr().write(data);
        }
        self.state.store(true, Ordering::SeqCst);
        unsafe { self.get_unchecked() }
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
        if !self.state.load(Ordering::SeqCst) {
            return None;
        }
        Some(unsafe { self.get_unchecked() })
    }
}
