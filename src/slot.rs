use crate::{assert_active_manager, ChannelType, Manager, Mode};
use std::{cell::RefCell, cmp::Ordering, fmt, rc::Rc};

/// List of [Node](crate::Node)s to a signal `T`.
///
/// A `slot` is a list in which one can store node handles.
///
/// ```
/// use revent::{Anchor, Grapher, Manager, Node, Slot};
/// use std::{cell::RefCell, rc::Rc};
///
/// trait BasicSignal {
///     fn basic(&mut self);
/// }
///
/// struct MyAnchor {
///     basic_slot: Slot<dyn BasicSignal>,
///     mng: Manager,
/// }
/// impl MyAnchor {
///     fn new() -> Self {
///         let mng = Manager::new();
///         Self {
///             basic_slot: Slot::new("basic_slot", &mng),
///             mng,
///         }
///     }
/// }
/// impl Anchor for MyAnchor {
///     fn manager(&self) -> &Manager {
///         &self.mng
///     }
/// }
///
/// // ---
///
/// struct MyNode;
/// impl Node<MyAnchor, ()> for MyNode {
///     fn register_emits(_: &MyAnchor) -> () { () }
///
///     fn register_listens(hub: &mut MyAnchor, item: Rc<RefCell<Self>>) {
///         hub.basic_slot.register(item);
///     }
///     const NAME: &'static str = "MyNode";
/// }
/// impl BasicSignal for MyNode {
///     fn basic(&mut self) {
///         println!("Hello from MyNode::basic");
///     }
/// }
///
/// // ---
///
/// let mut hub = MyAnchor::new();
/// let item = hub.subscribe(|_| MyNode);
/// hub.basic_slot.emit(|x| x.basic());
/// hub.unsubscribe(&item);
///
/// Grapher::new(hub.manager()).graph_to_file("target/slot-example.png").unwrap();
/// ```
pub struct Slot<T: ?Sized> {
    manager: Manager,
    name: &'static str,
    nodes: Rc<RefCell<Vec<Rc<RefCell<T>>>>>,
}

impl<T: ?Sized> Slot<T> {
    /// Create a new slot object.
    ///
    /// The manager is used to organize multiple single/slot objects and to ensure that there are no
    /// recursive (double mutable borrow) signal chains.
    ///
    /// `name` is used for error reporting and graph generation in [Manager].
    pub fn new(name: &'static str, manager: &Manager) -> Self {
        manager.ensure_new(name, ChannelType::Direct);
        Self {
            manager: manager.clone(),
            name,
            nodes: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Emit a signal on this signal slot.
    pub fn emit<F>(&mut self, mut caller: F)
    where
        F: FnMut(&mut T),
    {
        #[cfg(feature = "logging")]
        self.manager.log_emit(self.name);
        for item in self.nodes.borrow_mut().iter_mut() {
            #[cfg(feature = "logging")]
            self.manager.log_emit_on_item(item.clone(), self.name);
            let mut item = item.borrow_mut();
            caller(&mut *item);
        }
        #[cfg(feature = "logging")]
        self.manager.log_emit_end();
    }

    /// Sort the nodes to this slot.
    pub fn sort_by<F>(&mut self, mut compare: F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.nodes.borrow_mut().sort_by(|x, y| {
            let x = x.borrow();
            let y = y.borrow();
            compare(&*x, &*y)
        });
    }

    /// Add or remove a subscriber object to this slot.
    ///
    /// The action taken depends on whether [Anchor::subscribe](crate::Anchor::subscribe) or
    /// [Anchor::unsubscribe](crate::Anchor::unsubscribe) was called.
    ///
    /// When adding: pushes the item to the end of the list. See [sort_by](Slot::sort_by) if a different order is
    /// desired.
    /// When removing: `find`s the first matching instance and removes it.
    ///
    /// # Panics #
    ///
    /// Panics if called from [Anchor::unsubscribe](crate::Anchor::unsubscribe) while not being
    /// registered.
    ///
    /// Panics if called more than once for the same object from within
    /// [Anchor::subscribe](crate::Anchor::subscribe).
    pub fn register(&mut self, item: Rc<RefCell<T>>) {
        assert_active_manager(&self.manager);
        crate::STACK.with(|x| {
            let mode = x.borrow_mut().last().unwrap().0;
            match mode {
                Mode::Adding => {
                    self.manager.register_listen(self.name);
                    self.nodes.borrow_mut().push(item);
                }
                Mode::Removing => {
                    let mut subs = self.nodes.borrow_mut();
                    match subs
                        .iter()
                        .enumerate()
                        .find(|(_, value)| Rc::ptr_eq(&item, value))
                    {
                        Some((idx, _)) => {
                            subs.remove(idx);
                        }
                        None => {
                            panic!(
                                "revent: unable to deregister nonexistent item: {:?}",
                                self.name
                            );
                        }
                    }
                }
            }
        });
    }
}

impl<T: ?Sized> Clone for Slot<T> {
    /// Cloning is only valid from within an [Anchor::subscribe](crate::Anchor::subscribe) context.
    fn clone(&self) -> Self {
        assert_active_manager(&self.manager);
        self.manager.register_emit(self.name);
        Self {
            manager: self.manager.clone(),
            name: self.name,
            nodes: self.nodes.clone(),
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
            .field("nodes", &PointerWrapper(self.nodes.clone()))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Anchor, Manager, Node, Slot};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    #[should_panic(expected = "revent: signal modification outside of Anchor context")]
    fn using_signal_push_outside_subscribe() {
        trait Interface {}
        impl Interface for () {}

        let mut signal: Slot<dyn Interface> = Slot::new("signal", &Manager::new());

        signal.register(Rc::new(RefCell::new(())));
    }

    #[test]
    #[should_panic(expected = "revent: signal modification outside of Anchor context")]
    fn using_signal_clone_outside_subscribe() {
        trait Interface {}
        impl Interface for () {}

        let signal: Slot<dyn Interface> = Slot::new("signal", &Manager::new());

        let _ = signal.clone();
    }

    #[test]
    #[should_panic(expected = "revent: manager is different")]
    fn subscribing_with_different_manager() {
        trait Interface {}
        impl Interface for () {}

        // ---

        struct MyAnchor {
            signal_a: Slot<dyn Interface>,
            manager: Manager,
        }

        let mut hub = MyAnchor {
            signal_a: Slot::new("signal_a", &Manager::new()),
            manager: Manager::new(),
        };

        impl Anchor for MyAnchor {
            fn manager(&self) -> &Manager {
                &self.manager
            }
        }

        // ---

        struct MyEmitter;
        struct MyNode;
        impl Node<MyAnchor, MyEmitter> for MyNode {
            fn register_emits(_: &MyAnchor) -> MyEmitter {
                MyEmitter
            }
            fn register_listens(hub: &mut MyAnchor, item: Rc<RefCell<Self>>) {
                hub.signal_a.register(item);
            }
            const NAME: &'static str = "MyNode";
        }
        impl Interface for MyNode {}

        hub.subscribe(|_| MyNode);
    }

    #[test]
    #[should_panic(expected = "revent: name is already registered to this manager: \"signal\"")]
    fn double_subscription() {
        let mng = &Manager::new();

        Slot::<()>::new("signal", mng);
        Slot::<()>::new("signal", mng);
    }
}
