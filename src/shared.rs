use std::{cell::RefCell, marker::Unsize, ops::CoerceUnsized, rc::Rc};

/// An opaque struct containing a shared reference to a subscriber.
///
/// Used in
/// [Topic::subscribe](crate::Topic::subscribe) and provided in
/// [Subscriber::subscribe](crate::Subscriber::subscribe).
pub struct Shared<T: ?Sized>(
    /// Do not rely on this variable. It may change and has been marked undocumented. It is
    /// only used internally by this library and in the macro generated code.
    #[doc(hidden)]
    pub Rc<RefCell<T>>,
);

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
