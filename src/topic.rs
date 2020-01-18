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
    subscribers: Rc<RefCell<Vec<Rc<UnsafeCell<T>>>>>,
}

impl<T: 'static + ?Sized> Topic<T> {
    /// Emit an event into this topic to all subscribers.
    ///
    /// The `caller` variable is applied once to every single subscriber of this topic. Use this function to call the various methods on the subscribers.
    /// Subscribers are applied to `caller` in arbitrary order.
    pub fn emit(&self, mut caller: impl FnMut(&mut T)) {
        self.manager.borrow_mut().emitting(self.name);
        if !self.active {
            panic!("Topic is not active: {}", self.name);
        }
        if self.manager.borrow_mut().construction {
            panic!("Can not emit while an object is under construction");
        }
        for subscriber in self.subscribers.borrow_mut().iter() {
            caller(unsafe { &mut *subscriber.get() });
        }
    }

    /// Subscribe to this channel. Used only in
    /// [Subscriber::subscribe](crate::Subscriber::subscribe) to make a
    /// [Subscriber](crate::Subscriber) subscribe to hub topics.
    pub fn subscribe(&self, shared: Shared<T>) {
        self.manager.borrow_mut().subscribe_channel(self.name);
        self.subscribers.borrow_mut().push(shared.0);
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
            subscribers: Rc::new(RefCell::new(Vec::new())),
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
