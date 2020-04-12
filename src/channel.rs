use crate::Node;
use std::cmp::Ordering;

/// Container for multiple [Node]s.
///
/// ```
/// use revent::{Channel, Node};
///
/// let mut channel = Channel::new();
///
/// for number in 0..10 {
///     channel.insert(Node::new(number));
/// }
///
/// channel.emit(|x| {
///     println!("{}", x);
/// });
/// ```
pub struct Channel<T: ?Sized> {
    items: Vec<Node<T>>,
}

impl<T: ?Sized> Default for Channel<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> Channel<T> {
    /// Create a new channel.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Insert a node into this channel.
    ///
    /// Appends the node to the end of the channel.
    pub fn insert(&mut self, item: Node<T>) {
        self.items.push(item);
    }

    /// Remove a node from this channel if it exists.
    ///
    /// # Performance #
    ///
    /// Performs a linear scan and retains only those nodes that do not match.
    pub fn remove(&mut self, item: &Node<T>) {
        self.items.retain(|x| !Node::<T>::ptr_eq(item, x));
    }

    /// Apply a function to each item in this channel.
    pub fn emit(&self, mut handler: impl FnMut(&mut T)) {
        for item in self.items.iter() {
            item.emit(|x| {
                (handler)(x);
            });
        }
    }

    /// Apply a function to each item in this channel.
    ///
    /// Immutable version of [emit](Channel::emit).
    pub fn emit_ref(&self, mut handler: impl FnMut(&T)) {
        for item in self.items.iter() {
            item.emit_ref(|x| {
                (handler)(x);
            });
        }
    }

    /// Sort the nodes in this channel.
    pub fn sort_by<F>(&mut self, mut compare: F)
    where
        F: FnMut(&T, &T) -> Ordering,
    {
        self.items
            .sort_by(|a, b| a.emit_ref(|a| b.emit_ref(|b| (compare)(a, b))));
    }
}

impl<T: Ord + ?Sized> Channel<T> {
    /// Sort the channel.
    pub fn sort(&mut self) {
        self.sort_by(|a, b| a.cmp(b));
    }
}

#[cfg(test)]
mod tests {
    use super::{Channel, Node};

    #[test]
    fn duplicate_node_sort() {
        let node = Node::new(());
        let mut channel = Channel::new();
        channel.insert(node.clone());
        channel.insert(node);

        channel.sort();
    }
}
