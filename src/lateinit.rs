/// Provides an unsafe way to late-initialize static variables that will see a lot of use.
/// Basically a wrapper around Cell that only permits setting once and has debug support to ensure
/// this.

use core::{
    ops::Deref,
    cell::UnsafeCell,
    fmt::{
        Display,
        Debug,
        Formatter,
        Error as FmtError
    }
};

pub struct LateInit<T>(UnsafeCell<Option<T>>);

unsafe impl <T> Sync for LateInit<T> {}

impl <T> LateInit<T> {
    pub const fn new() -> Self {
        LateInit(UnsafeCell::new(None))
    }

    pub unsafe fn init(&self, value: T) {
        assert!((*self.0.get()).is_none(), "LateInit.init called more than once");
        *self.0.get() = Some(value);
    }
}

impl <T> Deref for LateInit<T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        debug_assert!(unsafe { *(&(*self.0.get()).is_some()) }, "LateInit used without initialization");
        (unsafe { &(*self.0.get()) }).as_ref().unwrap()
    }
}

impl <T: Debug> Debug for LateInit<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match unsafe { &(*self.0.get()) } {
            Some(ref x) => { x.fmt(f) },
            None => { write!(f, "<UNINITIALIZED>") },
        }
    }
}

impl <T: Display> Display for LateInit<T> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), FmtError> {
        match unsafe { &(*self.0.get()) } {
            Some(ref x) => { x.fmt(f) },
            None => { write!(f, "<UNINITIALIZED>") },
        }
    }
}
