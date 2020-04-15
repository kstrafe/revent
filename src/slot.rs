use crate::{Node, Trace};

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
    trace: Trace,
}

impl<T: ?Sized> Default for Slot<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: ?Sized> Slot<T> {
    /// Create a new slot.
    pub fn new() -> Self {
        Self {
            items: None,
            trace: Trace::empty(),
        }
    }

    /// Create a new channel with a trace object.
    pub fn new_with_trace(trace: impl Fn(usize) + 'static) -> Self {
        Self {
            items: None,
            trace: Trace::new(trace),
        }
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
    pub fn emit<R>(&self, handler: impl FnOnce(&mut T) -> R) -> R {
        self.trace.log();
        Trace::indent();

        let value = if let Some(value) = self.items.as_ref() {
            value.emit(|x| (handler)(x))
        } else {
            panic!("revent: emit: slot contains no element");
        };

        Trace::dedent();

        value
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[quickcheck_macros::quickcheck]
    fn emit_works(value: usize) {
        let mut slot = Slot::new();
        slot.insert(Node::new(value));

        let mut count = 0;
        slot.emit(|x| {
            count += 1;
            assert_eq!(*x, value);
        });
        assert_eq!(count, 1);
    }

    #[quickcheck_macros::quickcheck]
    fn emit_remove_emit(mut items: Vec<String>) {
        let mut slot = Slot::new();

        for item in items.drain(..) {
            let cloned = item.clone();
            let node = Node::new(item);
            slot.insert(node.clone());

            let mut count = 0;
            slot.emit(|x| {
                count += 1;
                assert_eq!(*x, cloned);
            });
            assert_eq!(count, 1);

            slot.remove();
        }
    }

    #[test]
    #[should_panic(expected = "revent: emit: slot contains no element")]
    fn emit_without_insert() {
        let slot = Slot::<()>::new();
        slot.emit(|_| {});
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
        let mut slot = Slot::new_with_trace(move |indent| {
            assert!(matches!(*capture.borrow(), None));
            *capture.borrow_mut() = Some(indent);
        });

        let capture = out.clone();
        slot.insert(Node::new_with_trace((), move |indent| {
            assert!(matches!(*capture.borrow(), Some(0)));
            *capture.borrow_mut() = Some(indent);
        }));

        slot.emit(|_| {});

        assert!(matches!(*out.borrow(), Some(1)));
    }
}
