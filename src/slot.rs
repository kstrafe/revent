use crate::Node;

/// Container for a single optional [Node].
///
/// Useful for providing a container that may be empty. If the item is always supposed to exist
/// then using [Node] directly is more useful.
/// ```
/// use revent::{Node, Slot};
///
/// let mut slot = Slot::new();
///
/// slot.insert(Node::new(123));
///
/// let result: i32 = slot.emit(|x| {
///     println!("{}", x);
///     *x + 1
/// });
///
/// println!("{}", result);
/// ```
pub struct Slot<T: ?Sized> {
    items: Option<Node<T>>,
}

impl<T: ?Sized> Default for Slot<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> Slot<T> {
    /// Create a new slot.
    pub fn new() -> Self {
        Self { items: None }
    }

    /// Insert a node into this slot.
    ///
    /// # Panics #
    ///
    /// Panics if there already exists a node in this slot.
    pub fn insert(&mut self, item: Node<T>) {
        self.items = Some(item);
    }

    /// Remove the currently held node from this slot.
    ///
    /// # Panics #
    ///
    /// Panics if there exists no node in this slot.
    pub fn remove(&mut self) -> Node<T> {
        self.items.take().unwrap()
    }

    /// Apply a function to the node in this slot.
    ///
    /// # Panics #
    ///
    /// Panics if there exists no node in this slot.
    pub fn emit<R>(&self, mut handler: impl FnMut(&mut T) -> R) -> R {
        self.items.as_ref().unwrap().emit(|x| (handler)(x))
    }

    /// Apply a function to the node in this slot.
    ///
    /// Immutable version of [emit](Slot::emit).
    ///
    /// # Panics #
    ///
    /// Panics if there exists no node in this slot.
    pub fn emit_ref<R>(&self, mut handler: impl FnMut(&T) -> R) -> R {
        self.items.as_ref().unwrap().emit_ref(|x| (handler)(x))
    }
}
