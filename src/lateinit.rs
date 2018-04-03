/// Provides an unsafe way to late-initialize static variables that will see a lot of use.
/// Basically a wrapper around Cell that only permits setting once and has debug support to ensure
/// this.

use core::{ops, cell::UnsafeCell};

pub struct LateInit<T: Sized> {
    data: UnsafeCell<T>,
    initialized: UnsafeCell<bool>,
}

unsafe impl <T: Sized> Sync for LateInit<T> {}

impl <T: Sized + Default> LateInit<T> {
    pub fn new() -> Self {
        unsafe {
            LateInit {
                data: UnsafeCell::new(T::default()),
                initialized: UnsafeCell::new(false),
            }
        }
    }

    pub unsafe fn init(&self, value: T) {
        assert!(!*(self.initialized.get()), "LateInit initialized more than once");
        *self.data.get() = value;
        *self.initialized.get() = false;
    }
}

impl <T: Sized> ops::Deref for LateInit<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        debug_assert!(unsafe { *(self.initialized.get()) }, "LateInit used without initialization");
        unsafe { self.data.get().as_ref().unwrap() }
    }
}
