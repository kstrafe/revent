use crate::{Manager, Shared};
use std::{cell::RefCell, rc::Rc};

/// An event channel for a certain type of [Subscriber](crate::Subscriber).
pub struct Topic<T: 'static + ?Sized>(Shared<InternalTopic<T>>);

struct InternalTopic<T: 'static + ?Sized> {
    manager: Shared<Manager>,
    name: &'static str,
    subscribers: Vec<Shared<T>>,
}

unsafe impl<T: Send + ?Sized> Send for Topic<T> {}

impl<T: 'static + ?Sized> Topic<T> {
    /// Emit an event into this topic to all subscribers.
    ///
    /// The `caller` variable is applied once to every single subscriber of this topic. Use this function to call the various methods on the subscribers.
    /// Subscribers are applied to `caller` in arbitrary order.
    pub fn emit(&mut self, mut caller: impl FnMut(&mut T)) {
        let internal = unsafe { &mut *(self.0).0.get() };
        unsafe { &mut *internal.manager.get() }.emitting(internal.name);
        for subscriber in internal.subscribers.iter() {
            caller(unsafe { &mut *subscriber.0.get() });
        }
    }

    /// Remove elements from a topic.
    ///
    /// If the closure returns true, then the element is removed. If the closure returns false, the
    /// element will remain in the topic.
    pub fn remove<F>(&mut self, mut caller: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let internal = unsafe { &mut *(self.0).0.get() };
        unsafe { &mut *internal.manager.get() }.emitting(internal.name);
        internal
            .subscribers
            .drain_filter(|subscriber| caller(unsafe { &mut *subscriber.0.get() }));
    }

    #[doc(hidden)]
    pub fn new(name: &'static str, manager: &Shared<Manager>) -> Self {
        Self(Shared::new(InternalTopic {
            manager: manager.clone(),
            name,
            subscribers: Vec::new(),
        }))
    }

    #[doc(hidden)]
    pub unsafe fn clone_activate(&self) -> Self {
        let internal = &mut *(self.0).0.get();
        (&mut *internal
                .manager
                .get())
            .activate_channel(internal.name);
        Self(self.0.clone())
    }

    #[doc(hidden)]
    pub unsafe fn subscribe(&mut self, shared: Shared<T>) {
        let internal = &mut *(self.0).0.get();
        (&mut *internal
            .manager
            .get())
            .subscribe_channel(internal.name);
        internal.subscribers.push(shared);
    }
}
