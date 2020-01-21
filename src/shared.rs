#![doc(hidden)]
use std::{cell::UnsafeCell, marker::Unsize, ops::CoerceUnsized, rc::Rc};

/// An opaque struct containing a shared reference to a subscriber.
///
/// For internal use only.
pub struct Shared<T: ?Sized>(pub(crate) Rc<UnsafeCell<T>>);

impl<T> Shared<T> {
    /// Create a new shared object.
    pub fn new(item: T) -> Self {
        Self(Rc::new(UnsafeCell::new(item)))
    }

    /// Get the raw pointer value.
    pub fn get(&self) -> *mut T {
        self.0.get()
    }

    /// Clone this shared object.
    ///
    /// Only intended to be used by the [hub] macro. Please DO NOT use this function as it might be
    /// removed or changed, or cause undefined behavior if used improperly.
    pub unsafe fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T, U> CoerceUnsized<Shared<U>> for Shared<T>
where
    T: Unsize<U> + ?Sized,
    U: ?Sized,
{
}

unsafe impl<T: Send> Send for Shared<T> {}
