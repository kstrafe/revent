use std::{cell::UnsafeCell, marker::Unsize, ops::CoerceUnsized, rc::Rc};

/// An opaque struct containing a shared reference to a subscriber.
pub struct Shared<T: ?Sized>(pub(crate) Rc<UnsafeCell<T>>);

impl<T> Shared<T> {
    #[doc(hidden)]
    pub fn new(item: T) -> Self {
        Self(Rc::new(UnsafeCell::new(item)))
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T, U> CoerceUnsized<Shared<U>> for Shared<T>
where
    T: Unsize<U> + ?Sized,
    U: ?Sized,
{
}
