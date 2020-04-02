use crate::{assert_active_manager, Manager, Mode};
use std::{cell::RefCell, cmp::Ordering, fmt, rc::Rc};

/// List of [Subscriber](crate::Subscriber)s to a signal `T`.
///
/// A `slot` is a list in which one can store subscriber handles.
pub struct Slot<T: ?Sized> {
    manager: Rc<RefCell<Manager>>,
    name: &'static str,
    subscribers: Rc<RefCell<Vec<Rc<RefCell<T>>>>>,
}

impl<T: ?Sized> Slot<T> {
    /// Create a new slot object.
    ///
    /// The manager is used to organize multiple single/slot objects and to ensure that there are no
    /// recursive (double mutable borrow) signal chains.
    ///
    /// `name` is used for error reporting and graph generation in [Manager].
    pub fn new(name: &'static str, manager: Rc<RefCell<Manager>>) -> Self {
        manager.borrow_mut().ensure_new(name);
        Self {
            manager,
            name,
            subscribers: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Emit a signal on this signal slot.
    pub fn emit<F>(&mut self, mut caller: F)
    where
        F: FnMut(&mut T),
    {
        for item in self.subscribers.borrow_mut().iter_mut() {
            let mut item = item.borrow_mut();
            caller(&mut *item);
        }
    }

    /// Continue emitting only if the caller returns `true`. Stops if `false`.
    pub fn emit_short<F>(&mut self, mut caller: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        for item in self.subscribers.borrow_mut().iter_mut() {
            let mut item = item.borrow_mut();
            if !caller(&mut *item) {
                break;
            }
        }
    }

    /// Sort the subscribers to this slot.
    pub fn sort_by<F>(&mut self, mut compare: F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.subscribers.borrow_mut().sort_by(|x, y| {
            let x = x.borrow();
            let y = y.borrow();
            compare(&*x, &*y)
        });
    }

    /// Add or remove a subscriber object to this slot.
    ///
    /// The action taken depends on whether [Node::subscribe](crate::Node::subscribe) or
    /// [Node::unsubscribe](crate::Node::unsubscribe) was called.
    ///
    /// When adding: pushes the item to the end of the list. See [sort_by](Slot::sort_by) if a different order is
    /// desired.
    /// When removing: `find`s the first matching instance and removes it.
    pub fn register(&mut self, item: Rc<RefCell<T>>) {
        assert_active_manager(&self.manager);
        let mut mng = self.manager.borrow_mut();
        crate::STACK.with(|x| {
            let mode = x.borrow_mut().last().unwrap().0;
            match mode {
                Mode::Adding => {
                    mng.register_subscribe(self.name);
                    self.subscribers.borrow_mut().push(item);
                }
                Mode::Removing => {
                    let mut subs = self.subscribers.borrow_mut();
                    let (idx, _) = subs
                        .iter()
                        .enumerate()
                        .find(|(_, value)| Rc::ptr_eq(&item, value))
                        .expect("unable to unsubscribe non-subscribed item");
                    subs.remove(idx);
                }
            }
        });
    }
}

impl<T: ?Sized> Clone for Slot<T> {
    /// Cloning is only valid from within a [Node::subscribe](crate::Node::subscribe) context.
    fn clone(&self) -> Self {
        assert_active_manager(&self.manager);
        self.manager.borrow_mut().register_emit(self.name);
        Self {
            manager: self.manager.clone(),
            name: self.name,
            subscribers: self.subscribers.clone(),
        }
    }
}

struct PointerWrapper<T: ?Sized>(Rc<RefCell<Vec<Rc<RefCell<T>>>>>);

impl<T: ?Sized> fmt::Debug for PointerWrapper<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.0.borrow().iter().map(|x| x.as_ptr()))
            .finish()
    }
}

impl<T: ?Sized> fmt::Debug for Slot<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Slot")
            .field("name", &self.name)
            .field("subscribers", &PointerWrapper(self.subscribers.clone()))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Manager, Named, Node, Slot, Subscriber};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    #[should_panic(expected = "revent signal modification outside of Node context")]
    fn using_signal_push_outside_subscribe() {
        trait Interface {}
        impl Interface for () {}

        let mut signal: Slot<dyn Interface> =
            Slot::new("signal", Rc::new(RefCell::new(Manager::default())));

        signal.register(Rc::new(RefCell::new(())));
    }

    #[test]
    #[should_panic(expected = "revent signal modification outside of Node context")]
    fn using_signal_clone_outside_subscribe() {
        trait Interface {}
        impl Interface for () {}

        let signal: Slot<dyn Interface> =
            Slot::new("signal", Rc::new(RefCell::new(Manager::default())));

        let _ = signal.clone();
    }

    #[test]
    #[should_panic(expected = "revent manager is different")]
    fn subscribing_with_different_manager() {
        trait Interface {}
        impl Interface for () {}

        // ---

        struct Hub {
            signal_a: Slot<dyn Interface>,
            manager: Rc<RefCell<Manager>>,
        }

        let mut hub = Hub {
            signal_a: Slot::new("signal_a", Rc::new(RefCell::new(Manager::default()))),
            manager: Rc::new(RefCell::new(Manager::default())),
        };

        impl Node for Hub {
            fn manager(&self) -> &Rc<RefCell<Manager>> {
                &self.manager
            }
        }

        // ---

        struct MyNode;
        impl From<&Hub> for MyNode {
            fn from(_: &Hub) -> MyNode {
                Self
            }
        }
        struct MySubscriber;
        impl Subscriber<Hub> for MySubscriber {
            type Input = ();
            type Node = MyNode;
            fn create(_: Self::Input, _: Self::Node) -> Self {
                Self
            }
            fn register(hub: &mut Hub, item: Rc<RefCell<Self>>) {
                hub.signal_a.register(item);
            }
        }
        impl Named for MySubscriber {
            const NAME: &'static str = "MySubscriber";
        }
        impl Interface for MySubscriber {}

        hub.subscribe::<MySubscriber>(());
    }

    #[test]
    #[should_panic(expected = "revent name is already registered to this manager: signal")]
    fn double_subscription() {
        let mng = Rc::new(RefCell::new(Manager::default()));

        Slot::<()>::new("signal", mng.clone());
        Slot::<()>::new("signal", mng);
    }
}
