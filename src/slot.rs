use crate::{assert_active_manager, Manager, Mode};
use std::{cell::RefCell, cmp::Ordering, fmt, rc::Rc};

/// List of [Node](crate::Node)s to a signal `T`.
///
/// A `slot` is a list in which one can store node handles.
pub struct Slot<T: ?Sized> {
    manager: Rc<RefCell<Manager>>,
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
    pub fn new(name: &'static str, manager: Rc<RefCell<Manager>>) -> Self {
        manager.borrow_mut().ensure_new(name);
        Self {
            manager,
            name,
            nodes: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Emit a signal on this signal slot.
    pub fn emit<F>(&mut self, mut caller: F)
    where
        F: FnMut(&mut T),
    {
        for item in self.nodes.borrow_mut().iter_mut() {
            let mut item = item.borrow_mut();
            caller(&mut *item);
        }
    }

    /// Continue emitting only if the caller returns `true`. Stops if `false`.
    pub fn emit_short<F>(&mut self, mut caller: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        for item in self.nodes.borrow_mut().iter_mut() {
            let mut item = item.borrow_mut();
            if !caller(&mut *item) {
                break;
            }
        }
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
        let mut mng = self.manager.borrow_mut();
        crate::STACK.with(|x| {
            let mode = x.borrow_mut().last().unwrap().0;
            match mode {
                Mode::Adding => {
                    mng.register_subscribe(self.name);
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
        self.manager.borrow_mut().register_emit(self.name);
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

        let mut signal: Slot<dyn Interface> =
            Slot::new("signal", Rc::new(RefCell::new(Manager::default())));

        signal.register(Rc::new(RefCell::new(())));
    }

    #[test]
    #[should_panic(expected = "revent: signal modification outside of Anchor context")]
    fn using_signal_clone_outside_subscribe() {
        trait Interface {}
        impl Interface for () {}

        let signal: Slot<dyn Interface> =
            Slot::new("signal", Rc::new(RefCell::new(Manager::default())));

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
            manager: Rc<RefCell<Manager>>,
        }

        let mut hub = MyAnchor {
            signal_a: Slot::new("signal_a", Rc::new(RefCell::new(Manager::default()))),
            manager: Rc::new(RefCell::new(Manager::default())),
        };

        impl Anchor for MyAnchor {
            fn manager(&self) -> &Rc<RefCell<Manager>> {
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
        let mng = Rc::new(RefCell::new(Manager::default()));

        Slot::<()>::new("signal", mng.clone());
        Slot::<()>::new("signal", mng);
    }
}