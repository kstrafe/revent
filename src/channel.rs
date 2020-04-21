use crate::{Node, Trace};
use isize_vec::IsizeVec;

/// Container for multiple [Node]s.
///
/// ```
/// use revent::{Channel, Node};
///
/// let mut channel = Channel::new();
///
/// for number in 0..10 {
///     channel.insert(0, Node::new(number));
/// }
///
/// channel.emit(|x| {
///     println!("{}", x);
/// });
/// ```
pub struct Channel<T: ?Sized> {
    items: IsizeVec<Node<T>>,
    trace: Trace,
}

impl<T: ?Sized> Default for Channel<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> Channel<T> {
    /// Create a new channel.
    pub fn new() -> Self {
        Self {
            items: IsizeVec::default(),
            trace: Trace::empty(),
        }
    }

    /// Create a new channel with a trace object.
    pub fn new_with_trace(trace: impl Fn(usize) + 'static) -> Self {
        Self {
            items: IsizeVec::default(),
            trace: Trace::new(trace),
        }
    }

    /// Insert a node into this channel.
    ///
    /// The value `relative` indicates where the node will be put in the list relative to other
    /// nodes. If two nodes have the same `relative` value, then the node will be prepended if it
    /// is signed, and appended if unsigned.
    pub fn insert(&mut self, relative: isize, item: Node<T>) {
        self.items.insert(relative, item);
    }

    /// Remove all occurrences of a node from this channel.
    ///
    /// # Performance #
    ///
    /// Performs a linear scan and retains only those nodes that do not match.
    pub fn remove(&mut self, item: &Node<T>) {
        self.items.retain(|x| !Node::<T>::ptr_eq(item, x));
    }

    /// Apply a function to each item in this channel.
    pub fn emit(&self, mut handler: impl FnMut(&mut T)) {
        self.trace.log();
        Trace::indent();

        for item in self.items.iter() {
            item.emit(|x| {
                (handler)(x);
            });
        }

        Trace::dedent();
    }
}

#[cfg(test)]
mod tests {
    use super::{Channel, Node};

    #[test]
    fn removing_considers_order() {
        let mut channel = Channel::new();
        let node = Node::new(());
        channel.insert(0, node.clone());
        channel.remove(&node);

        channel.insert(1, Node::new(()));
    }

    #[quickcheck_macros::quickcheck]
    fn inserting_appends_or_prepends(relative: isize, nodes: usize) {
        let mut channel = Channel::new();

        for node in 0..nodes {
            channel.insert(relative, Node::new(node));
        }

        if relative >= 0 {
            let mut value = 0;
            channel.emit(|x| {
                assert_eq!(value, *x);
                value += 1;
            });
            assert_eq!(value, nodes);
        } else {
            let mut value = nodes;
            channel.emit(|x| {
                value -= 1;
                assert_eq!(value, *x);
            });
            assert_eq!(value, 0);
        }
    }

    #[test]
    fn basic() {
        let mut channel = Channel::new();

        let node = Node::new(0);
        channel.insert(0, node.clone());
        channel.insert(1, Node::new(1));

        let mut number = 0;
        channel.emit(|x| {
            assert_eq!(*x, number);
            number += 1;
        });
        assert_eq!(number, 2);

        channel.remove(&node);

        let mut number = 1;
        channel.emit(|x| {
            assert_eq!(*x, number);
            number += 1;
        });
        assert_eq!(number, 2);
    }

    #[test]
    fn haystack() {
        let mut channel = Channel::new();

        let node = Node::new(0);
        for _ in 0..10 {
            channel.insert(0, node.clone());
        }
        for _ in 0..10 {
            channel.insert(2, node.clone());
        }

        channel.insert(1, Node::new(1));

        channel.remove(&node);

        let mut count = 0;
        channel.emit(|x| {
            assert_eq!(*x, 1);
            count += 1;
        });
        assert_eq!(count, 1);
    }
}

#[cfg(all(test, feature = "trace"))]
mod trace_tests {
    use crate::*;
    use std::{cell::RefCell, rc::Rc};

    #[test]
    fn tracing() {
        let out = Rc::new(RefCell::new(None));

        let capture = out.clone();
        let mut channel = Channel::new_with_trace(move |indent| {
            assert!(matches!(*capture.borrow(), None));
            *capture.borrow_mut() = Some(indent);
        });

        let capture = out.clone();
        channel.insert(
            0,
            Node::new_with_trace((), move |indent| {
                assert!(matches!(*capture.borrow(), Some(0)));
                *capture.borrow_mut() = Some(indent);
            }),
        );

        channel.emit(|_| {});

        assert!(matches!(*out.borrow(), Some(1)));
    }
}
