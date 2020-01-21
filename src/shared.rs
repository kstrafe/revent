#![doc(hidden)]
use std::{cell::UnsafeCell, marker::Unsize, ops::CoerceUnsized, rc::Rc};

pub struct Shared<T: ?Sized>(pub(crate) Rc<UnsafeCell<T>>);

impl<T> Shared<T> {
    pub fn new(item: T) -> Self {
        Self(Rc::new(UnsafeCell::new(item)))
    }

    pub fn get(&self) -> *mut T {
        self.0.get()
    }

    /// # Safety #
    /// Only used internally.
    pub unsafe fn clone_shared(&self) -> Self {
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
