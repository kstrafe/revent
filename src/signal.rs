use crate::Manager;
use std::{cell::RefCell, cmp::Ordering, rc::Rc};

/// Signal of interest, contains subscribers.
///
/// Signals are channels on which we can emit events synchronously. This is done by [Signal::emit].
/// Signals are not be created by the user directly, but are rather created via the [hub] and [node]
/// macros.
pub struct Signal<T: ?Sized>(Rc<InternalSignal<T>>);

impl<T: ?Sized> Signal<T> {
    /// Access all subscribers and apply a closure to each.
    ///
    /// Subscribers are iterated in subscription order.
    pub fn emit<F>(&mut self, mut caller: F)
    where
        F: FnMut(&mut T),
    {
        for item in self.0.subscribers.borrow_mut().iter_mut() {
            let mut item = item.borrow_mut();
            caller(&mut *item);
        }
    }

    /// Remove subscribers from a topic given a predicate. True removes the item.
    ///
    /// Removing items does not perturb relative ordering.
    pub fn remove<F>(&mut self, mut predicate: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        self.0.subscribers.borrow_mut().drain_filter(|item| {
            let mut item = item.borrow_mut();
            predicate(&mut *item)
        });
    }

    /// Sorts the topic with a comparator function.
    pub fn sort_by<F>(&mut self, mut compare: F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.0.subscribers.borrow_mut().sort_by(|x, y| {
            let x = x.borrow();
            let y = y.borrow();
            compare(&*x, &*y)
        });
    }

    #[doc(hidden)]
    pub fn new(name: &'static str, manager: Rc<RefCell<Manager>>) -> Self {
        Self(Rc::new(InternalSignal::new(name, manager)))
    }

    #[doc(hidden)]
    pub fn internal_clone(&self) -> Self {
        self.0.manager.borrow_mut().register_emit(self.0.name);
        Self(self.0.clone())
    }

    #[doc(hidden)]
    pub fn insert(&self, item: Rc<RefCell<T>>) {
        self.0.manager.borrow_mut().register_subscribe(self.0.name);
        self.0.subscribers.borrow_mut().push(item);
    }
}

struct InternalSignal<T: ?Sized> {
    pub manager: Rc<RefCell<Manager>>,
    pub name: &'static str,
    pub subscribers: RefCell<Vec<Rc<RefCell<T>>>>,
}

impl<T: ?Sized> InternalSignal<T> {
    pub fn new(name: &'static str, manager: Rc<RefCell<Manager>>) -> Self {
        InternalSignal {
            manager,
            name,
            subscribers: RefCell::new(Vec::new()),
        }
    }
}
