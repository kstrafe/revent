use crate::{Manager, Mode};
use std::{cell::RefCell, rc::Rc};

/// A collection of [Slot](crate::Slot)s and [Single](crate::Single)s to which [Subscriber]s can subscribe.
///
/// Hubs must be organized by a [Manager], this is done by implementing a data structure
/// containing various slots together with this trait.
///
/// A typical implementation may look like this:
/// ```
/// use revent::{Anchor, Manager, Slot};
/// use std::{cell::RefCell, rc::Rc};
///
/// struct MySlots {
///     a: Slot<()>,
///     b: Slot<()>,
///     // more slots...
///     manager: Rc<RefCell<Manager>>,
/// }
///
/// impl Anchor for MySlots {
///     fn manager(&self) -> &Rc<RefCell<Manager>> {
///         &self.manager
///     }
/// }
/// ```
pub trait Anchor
where
    Self: Sized,
{
    /// Get the [Manager] of this hub.
    fn manager(&self) -> &Rc<RefCell<Manager>>;

    /// Add a subscriber to this hub.
    ///
    /// Uses [Subscriber::register] to figure out which slots to attach to.
    fn subscribe<T>(&mut self, input: T::Input) -> Rc<RefCell<T>>
    where
        T: Named + Subscriber<Self>,
        T::Outputs: for<'a> From<&'a Self>,
    {
        let manager = self.manager().clone();
        crate::STACK.with(|x| {
            x.borrow_mut().push((Mode::Adding, manager.clone()));
        });

        manager.borrow_mut().prepare_construction(T::NAME);

        let hub = T::Outputs::from(self);
        let item = Rc::new(RefCell::new(T::create(input, hub)));
        T::register(self, item.clone());

        manager.borrow_mut().finish_construction();
        crate::STACK.with(|x| {
            x.borrow_mut().pop();
        });
        item
    }

    /// Remove a subscriber from this hub.
    ///
    /// Uses [Subscriber::register] to figure out which slots to detach from.
    fn unsubscribe<T>(&mut self, input: &Rc<RefCell<T>>)
    where
        T: Subscriber<Self>,
    {
        let manager = self.manager().clone();
        crate::STACK.with(|x| {
            x.borrow_mut().push((Mode::Removing, manager.clone()));
        });

        T::register(self, input.clone());

        crate::STACK.with(|x| {
            x.borrow_mut().pop();
        });
    }
}

/// Describes a subscriber that can subscribe to [Hub].
/// ```
/// use revent::{Anchor, Manager, Named, Null, Slot, Subscriber};
/// use std::{cell::RefCell, rc::Rc};
///
/// trait A {}
///
/// struct MySlots {
///     a: Slot<A>,
///     manager: Rc<RefCell<Manager>>,
/// }
///
/// impl Anchor for MySlots {
///     fn manager(&self) -> &Rc<RefCell<Manager>> {
///         &self.manager
///     }
/// }
///
/// // ---
///
/// struct MyNode;
///
/// impl Subscriber<MySlots> for MyNode {
///     type Input = ();
///     type Outputs = Null;
///
///     fn create(input: Self::Input, node: Self::Outputs) -> Self {
///         Self
///     }
///
///     fn register(slots: &mut MySlots, item: Rc<RefCell<Self>>) {
///         slots.a.register(item);
///     }
///
/// }
/// impl Named for MyNode {
///     const NAME: &'static str = "MyNode";
/// }
///
/// impl A for MyNode {}
/// ```
pub trait Subscriber<N: Anchor> {
    /// The type of input used to construct an instance of itself.
    type Input;
    /// The type of the node it uses to further send signals to other [Slot](crate::Slot)s.
    ///
    /// May be [Null](crate::Null) if no further signals are sent from this subscriber.
    type Outputs;
    /// Create an instance of itself.
    fn create(input: Self::Input, hub: Self::Outputs) -> Self;
    /// Register to various channels inside a [Anchor].
    ///
    /// Note that this function is used for both [subscribe](Anchor::subscribe) as well as
    /// [unsubscribe](Anchor::unsubscribe). So this function ought to not depend on the state of the
    /// item. If it does, then a subscriber may still be subscribed to some channels after
    /// `unsubscribe` has been called.
    fn register(hub: &mut N, item: Rc<RefCell<Self>>);
}

/// Attach a name to a type.
pub trait Named {
    /// Unique name of the subscriber.
    ///
    /// Used for figuring out recursions and graphing channel dependencies.
    const NAME: &'static str;
}
