use crate::{Event, Notifiable, TypedBinarySystem};
use std::ops::{Deref, DerefMut};

mod guard;
pub use guard::NotifierGuard;

/// Wrapper structure for notifiers.
///
/// When a structure sends a notification, it must be split out from the tree structure it
/// originates from.
pub struct Notifier<T: Notifiable>(Option<T>);

impl<T: Notifiable> Notifier<T> {
    /// Create a new [Notifier].
    pub fn new(datum: T) -> Self {
        Self(Some(datum))
    }

    fn get(&mut self) -> T {
        self.0.take().unwrap()
    }

    fn set(&mut self, datum: T) {
        self.0 = Some(datum);
    }

    /// Access the notifier and turn the structure it is part of into a system.
    pub fn guard<'a, 'b, N: Notifiable, F: FnMut(&mut N) -> &mut Notifier<T>>(
        this: &'a mut N,
        mut accessor: F,
        system: &'b mut dyn Notifiable,
    ) -> NotifierGuard<'a, 'b, N, T, F> {
        let split = accessor(this).get();

        NotifierGuard {
            accessor,
            split: Some(split),
            system: TypedBinarySystem((this, system)),
        }
    }
}

impl<T: Notifiable> Deref for Notifier<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<T: Notifiable> DerefMut for Notifier<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}

impl<T: Notifiable> Notifiable for Notifier<T> {
    fn event(&mut self, event: &dyn Event, system: &mut dyn Notifiable) {
        if let Some(ref mut item) = self.0 {
            item.event(event, system);
        }
    }
}
