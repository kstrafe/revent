use std::{cell::UnsafeCell, marker::Unsize, ops::CoerceUnsized, rc::Rc};

/// An opaque struct containing a shared reference to a subscriber.
///
/// Used in
/// [Topic::subscribe](crate::Topic::subscribe) and provided in
/// [Subscriber::subscribe](crate::Subscriber::subscribe).
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
