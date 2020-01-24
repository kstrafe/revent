use std::{cell::RefCell, rc::Rc};

/// Describes how a type must insert itself into signals.
#[doc(hidden)]
pub trait Selfscriber<T> {
    /// Name of the struct implementing [Selfscriber].
    ///
    /// Implemented bv the [node] macro.
    fn name() -> &'static str;
    /// Inserts `item` in various signals in `T`.
    fn selfscribe(holder: &T, item: Rc<RefCell<Self>>);
}

/// Describes which internal node is used for `Self`.
#[doc(hidden)]
pub trait Nodified {
    /// Intermediate signalling node type. Must be generated from [node].
    type Node;
}

/// Describes how to build an object.
pub trait Subscriber: Nodified {
    /// Construction arguments to `Self`.
    type Input;
    /// Build an instance of `Self`.
    fn build(node: Self::Node, input: Self::Input) -> Self;
}
