use crate::{Manager, Shared};
use std::{
    cell::{RefCell, UnsafeCell},
    rc::Rc,
};

/// An event channel for a certain class of subscribers.
pub struct Topic<T: 'static + ?Sized> {
    active: bool,
    manager: Rc<RefCell<Manager>>,
    name: &'static str,
    subscribers: Rc<UnsafeCell<Vec<Shared<T>>>>,
}

impl<T: 'static + ?Sized> Topic<T> {
    /// Emit an event into this topic to all subscribers.
    ///
    /// The `caller` variable is applied once to every single subscriber of this topic. Use this function to call the various methods on the subscribers.
    /// Subscribers are applied to `caller` in arbitrary order.
    pub fn emit(&mut self, mut caller: impl FnMut(&mut T)) {
        self.manager.borrow_mut().emitting(self.name);
        if !self.active {
            panic!("Topic is not active (emit): {}", self.name);
        }
        if self.manager.borrow_mut().construction {
            panic!("Can not emit while an object is under construction");
        }
        for subscriber in unsafe { &mut *self.subscribers.get() }.iter() {
            caller(unsafe { &mut *subscriber.0.get() });
        }
    }

    /// Subscribe to this channel. Used only in
    /// [Subscriber::subscribe](crate::Subscriber::subscribe) to make a
    /// [Subscriber](crate::Subscriber) subscribe to hub topics.
    pub fn subscribe(&mut self, shared: Shared<T>) {
        self.manager.borrow_mut().subscribe_channel(self.name);
        unsafe { &mut *self.subscribers.get() }.push(shared);
    }

    /// Remove elements from a topic.
    ///
    /// If the closure returns true, then the element is removed. If the closure returns false, the
    /// element will remain in the topic.
    pub fn filter(&mut self, mut caller: impl FnMut(&mut T) -> bool) {
        self.manager.borrow_mut().emitting(self.name);
        if !self.active {
            panic!("Topic is not active (filter): {}", self.name);
        }
        if self.manager.borrow_mut().construction {
            panic!("Can not filter while an object is under construction");
        }
        unsafe { &mut *self.subscribers.get() }
            .drain_filter(|subscriber| caller(unsafe { &mut *subscriber.0.get() }));
    }

    /// Activate this channel instance.
    ///
    /// When inside [Subsciber::build](crate::Subscriber::build) one must activate each
    /// channel which one wishes to emit events onto for that subscriber.
    pub fn activate(&mut self) {
        if !self.manager.borrow_mut().construction {
            panic!("Activating channel outside of construction context");
        }
        if self.active {
            panic!("Channel is already active");
        }
        self.manager.borrow_mut().activate_channel(self.name);
        self.active = true;
    }

    #[doc(hidden)]
    pub fn new(name: &'static str, manager: &Rc<RefCell<Manager>>) -> Self {
        Self {
            active: true,
            manager: manager.clone(),
            name,
            subscribers: Rc::new(UnsafeCell::new(Vec::new())),
        }
    }

    #[doc(hidden)]
    pub fn clone_deactivate(&self) -> Self {
        Self {
            name: self.name,
            active: false,
            manager: self.manager.clone(),
            subscribers: self.subscribers.clone(),
        }
    }
}
