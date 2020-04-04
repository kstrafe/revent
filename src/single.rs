use crate::{assert_active_manager, Manager, Mode};
use std::{cell::RefCell, fmt, mem::replace, rc::Rc};

/// Single [Subscriber](crate::Subscriber) to a signal `T`.
///
/// A `single` is a container for a single subscriber. It ensures that a single subscriber always
/// exists, panicking if not present on access. In addition, no more than a single subscriber may
/// subscribe to this container at a time.
pub struct Single<T: ?Sized> {
    manager: Rc<RefCell<Manager>>,
    name: &'static str,
    subscriber: Rc<RefCell<Option<Rc<RefCell<T>>>>>,
}

impl<T: ?Sized> Single<T> {
    /// Create a new single object.
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
            subscriber: Rc::new(RefCell::new(None)),
        }
    }

    /// Emit a signal on this signal single.
    ///
    /// Panics if no subscriber is subscribed to this single.
    pub fn emit<F, R>(&mut self, mut caller: F) -> R
    where
        F: FnMut(&mut T) -> R,
    {
        let mut item = self.subscriber.borrow_mut();

        if let Some(item) = &mut *item {
            caller(&mut item.borrow_mut())
        } else {
            panic!("revent no subscriber in {:?}", self.name)
        }
    }

    /// Add or remove a subscriber object to this single.
    ///
    /// The action taken depends on whether [Node::subscribe](crate::Node::subscribe) or
    /// [Node::unsubscribe](crate::Node::unsubscribe) was called.
    pub fn register(&mut self, item: Rc<RefCell<T>>) {
        assert_active_manager(&self.manager);
        let mut mng = self.manager.borrow_mut();
        crate::STACK.with(|x| {
            let mode = x.borrow_mut().last().unwrap().0;
            match mode {
                Mode::Adding => {
                    mng.register_subscribe(self.name);
                    assert!(
                        replace(&mut *self.subscriber.borrow_mut(), Some(item)).is_none(),
                        "revent unable to register subscription to item twice: {:?}",
                        self.name
                    );
                }
                Mode::Removing => {
                    assert!(
                        replace(&mut *self.subscriber.borrow_mut(), Some(item)).is_some(),
                        "revent unable to deregister non-existent item: {:?}",
                        self.name
                    );
                }
            }
        });
    }
}

impl<T: ?Sized> Clone for Single<T> {
    /// Cloning is only valid from within a [Node::subscribe](crate::Node::subscribe) context.
    fn clone(&self) -> Self {
        assert_active_manager(&self.manager);
        self.manager.borrow_mut().register_emit(self.name);
        Self {
            manager: self.manager.clone(),
            name: self.name,
            subscriber: self.subscriber.clone(),
        }
    }
}

struct PointerWrapper<T: ?Sized>(Rc<RefCell<Option<Rc<RefCell<T>>>>>);

impl<T: ?Sized> fmt::Debug for PointerWrapper<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let item = self.0.borrow();
        f.debug_list()
            .entries(item.as_ref().map(|x| x.as_ptr()))
            .finish()
    }
}

impl<T: ?Sized> fmt::Debug for Single<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Single")
            .field("name", &self.name)
            .field("subscribers", &PointerWrapper(self.subscriber.clone()))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Anchor, Manager, Named, Single, Subscriber};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    #[should_panic(expected = "revent signal modification outside of Hub context")]
    fn using_signal_push_outside_subscribe() {
        trait Interface {}
        impl Interface for () {}

        let mut signal: Single<dyn Interface> =
            Single::new("signal", Rc::new(RefCell::new(Manager::default())));

        signal.register(Rc::new(RefCell::new(())));
    }

    #[test]
    #[should_panic(expected = "revent signal modification outside of Hub context")]
    fn using_signal_clone_outside_subscribe() {
        trait Interface {}
        impl Interface for () {}

        let signal: Single<dyn Interface> =
            Single::new("signal", Rc::new(RefCell::new(Manager::default())));

        let _ = signal.clone();
    }

    #[test]
    #[should_panic(expected = "revent manager is different")]
    fn subscribing_with_different_manager() {
        trait Interface {}
        impl Interface for () {}

        // ---

        struct Hub {
            signal_a: Single<dyn Interface>,
            manager: Rc<RefCell<Manager>>,
        }

        let mut hub = Hub {
            signal_a: Single::new("signal_a", Rc::new(RefCell::new(Manager::default()))),
            manager: Rc::new(RefCell::new(Manager::default())),
        };

        impl Anchor for Hub {
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
            type Outputs = MyNode;
            fn create(_: Self::Input, _: Self::Outputs) -> Self {
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
    #[should_panic(expected = "revent unable to register subscription to item twice: \"signal_a\"")]
    fn double_single_registered() {
        trait Interface {}
        impl Interface for () {}

        // ---

        struct Hub {
            signal_a: Single<dyn Interface>,
            manager: Rc<RefCell<Manager>>,
        }

        let mut hub = {
            let manager = Rc::new(RefCell::new(Manager::default()));
            Hub {
                signal_a: Single::new("signal_a", manager.clone()),
                manager,
            }
        };

        impl Anchor for Hub {
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
            type Outputs = MyNode;
            fn create(_: Self::Input, _: Self::Outputs) -> Self {
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
        hub.subscribe::<MySubscriber>(());
    }

    #[test]
    #[should_panic(expected = "revent no subscriber in \"signal_a\"")]
    fn emit_on_empty_single() {
        trait Interface {}
        impl Interface for () {}

        // ---

        struct Hub {
            signal_a: Single<dyn Interface>,
            manager: Rc<RefCell<Manager>>,
        }

        let mut hub = {
            let manager = Rc::new(RefCell::new(Manager::default()));
            Hub {
                signal_a: Single::new("signal_a", manager.clone()),
                manager,
            }
        };

        impl Anchor for Hub {
            fn manager(&self) -> &Rc<RefCell<Manager>> {
                &self.manager
            }
        }

        // ---

        hub.signal_a.emit(|_| {});
    }

    #[test]
    #[should_panic(expected = "revent name is already registered to this manager: signal")]
    fn double_subscription() {
        let mng = Rc::new(RefCell::new(Manager::default()));

        Single::<()>::new("signal", mng.clone());
        Single::<()>::new("signal", mng);
    }
}
