use crate::{Manager, Mode};
use std::{cell::RefCell, rc::Rc};

/// A collection of channels to which [Node]s can [subscribe](Anchor::subscribe).
///
/// Anchors must be organized by a [Manager], this is done by implementing a data structure
/// containing various channels together with this trait.
///
/// A typical implementation may look like this:
/// ```
/// use revent::{Anchor, feed::Feed, Manager, Single, Slot};
/// use std::{cell::RefCell, rc::Rc};
///
/// struct MyAnchor {
///     a: Slot<()>,
///     b: Single<()>,
///     c: Feed<()>,
///     // more channels...
///     manager: Manager,
/// }
///
/// impl Anchor for MyAnchor {
///     fn manager(&self) -> &Manager {
///         &self.manager
///     }
/// }
/// ```
pub trait Anchor
where
    Self: Sized,
{
    /// Get the [Manager] of this anchor.
    fn manager(&self) -> &Manager;

    /// Add a node to this anchor.
    ///
    /// Uses [Node::register_listens] to figure out which slots to attach to.
    /// [Node::register_emits] is used to construct a struct that is given to `create`.
    fn subscribe<R, T, F>(&mut self, create: F) -> Rc<RefCell<T>>
    where
        T: Node<Self, R>,
        F: FnOnce(R) -> T,
    {
        let manager = self.manager().clone();
        crate::STACK.with(|x| {
            x.borrow_mut().push((Mode::Adding, manager.clone()));
        });

        manager.prepare_construction(T::NAME);

        let register_emits = T::register_emits(self);
        let item = Rc::new(RefCell::new(create(register_emits)));
        T::register_listens(self, item.clone());

        manager.finish_construction();
        crate::STACK.with(|x| {
            x.borrow_mut().pop();
        });
        item
    }

    /// Remove a node from this anchor.
    ///
    /// Uses [Node::register_listens] to figure out which slots to detach from.
    fn unsubscribe<T, R>(&mut self, input: &Rc<RefCell<T>>)
    where
        T: Node<Self, R>,
    {
        let manager = self.manager();
        crate::STACK.with(|x| {
            x.borrow_mut().push((Mode::Removing, manager.clone()));
        });

        T::register_listens(self, input.clone());

        crate::STACK.with(|x| {
            x.borrow_mut().pop();
        });
    }
}

/// Describes a subscriber that can subscribe to [Anchor].
/// ```
/// use revent::{Anchor, Manager, Slot, Node};
/// use std::{cell::RefCell, rc::Rc};
///
/// trait A {}
///
/// struct MyAnchor {
///     a: Slot<dyn A>,
///     manager: Manager,
/// }
///
/// impl Anchor for MyAnchor {
///     fn manager(&self) -> &Manager {
///         &self.manager
///     }
/// }
///
/// // ---
///
/// struct MyNode;
///
/// impl Node<MyAnchor, ()> for MyNode {
///     fn register_emits(_: &MyAnchor) -> () { () }
///
///     fn register_listens(slots: &mut MyAnchor, item: Rc<RefCell<Self>>) {
///         slots.a.register(item);
///     }
///
///     const NAME: &'static str = "MyNode";
/// }
///
/// impl A for MyNode {}
/// ```
pub trait Node<A: Anchor, T> {
    /// Create an object containing clones of the anchor's signals.
    ///
    /// Typically constructs a struct which is stored in `Self`. This struct is given
    /// [Anchor::subscribe]'s create function.
    /// May return `()` if no further signals are sent from this node.
    fn register_emits(anchor: &A) -> T;
    /// Register to various channels inside an [Anchor].
    ///
    /// Note that this function is used for both [subscribe](Anchor::subscribe) as well as
    /// [unsubscribe](Anchor::unsubscribe). So this function ought to not depend on the state of the
    /// item. If it does, then a node may still be subscribed to some channels after
    /// `unsubscribe` has been called.
    fn register_listens(anchor: &mut A, item: Rc<RefCell<Self>>);
    /// Unique name of the node.
    ///
    /// Used for figuring out recursions and graphing channel dependencies.
    const NAME: &'static str;
}
