use crate::{borrow_mut, is_borrowed, unborrow_mut, BorrowFlag, Trace, STACK};
use std::{
    cell::{Cell, UnsafeCell},
    marker::Unsize,
    mem,
    ops::CoerceUnsized,
    rc::Rc,
};

/// Node containing arbitrary data.
///
/// Ensures that no double-mutable borrows exist and allows the contained item to
/// [Suspend](crate::Suspend) itself.
///
/// Node is fundamentally the same as [RefCell](std::cell::RefCell), but does one more thing:
/// it allows suspension of the last emitted node by using its `&mut`. Suspending allows the
/// node to be reborrowed without aliasing.
pub struct Node<T: ?Sized> {
    item: Rc<(Cell<BorrowFlag>, UnsafeCell<T>)>,
    size: usize,
    trace: Trace,
}

impl<T, U> CoerceUnsized<Node<U>> for Node<T>
where
    T: Unsize<U> + ?Sized,
    U: ?Sized,
{
}

impl<T> Clone for Node<T> {
    fn clone(&self) -> Self {
        Self {
            item: self.item.clone(),
            size: self.size,
            trace: self.trace.clone(),
        }
    }
}

impl<T> Node<T> {
    /// Create a new node.
    pub fn new(item: T) -> Self {
        Self {
            item: Rc::new((Cell::new(0), UnsafeCell::new(item))),
            size: mem::size_of::<T>(),
            trace: Trace::empty(),
        }
    }

    /// Create a new node with a tracing function.
    ///
    /// The trace function is called with the current indentation value. It can be used to log
    /// calls to nodes and see how various nodes call upon other nodes.
    ///
    /// Requires the `trace` feature to be enabled to actually use the `trace` function.
    pub fn new_with_trace(item: T, trace: impl Fn(usize) + 'static) -> Self {
        Self {
            item: Rc::new((Cell::new(0), UnsafeCell::new(item))),
            size: mem::size_of::<T>(),
            trace: Trace::new(trace),
        }
    }
}

impl<T: ?Sized> Node<T> {
    /// Acquire a `&mut` to the contents of the node and allow it to [Suspend](crate::Suspend) itself.
    ///
    /// ```
    /// use revent::Node;
    ///
    /// let node = Node::new(123);
    ///
    /// node.emit(|x| {
    ///     *x = 456;
    ///     println!("{}", x);
    /// });
    /// ```
    ///
    /// # Panics #
    ///
    /// Panics if the node has already been accessed without being suspended.
    ///
    /// ```should_panic
    /// use revent::Node;
    /// let node = Node::new(123);
    ///
    /// node.emit(|_| {
    ///     node.emit(|_| {});
    /// });
    /// ```
    ///
    /// The mitigation is to call suspend on the `emit` lambda argument.
    /// ```
    /// use revent::{Node, Suspend};
    /// let node = Node::new(123);
    ///
    /// node.emit(|x| {
    ///     x.suspend(|| {
    ///         node.emit(|_| {});
    ///     });
    /// });
    /// ```
    pub fn emit<F: FnOnce(&mut T) -> R, R>(&self, handler: F) -> R {
        self.trace.log();

        if is_borrowed(self.flag()) {
            panic!("revent: emit: accessing already borrowed item");
        }
        borrow_mut(self.flag());

        STACK.with(|x| {
            // unsafe: We know there exist no other borrows of `STACK`. It is _never_ borrowed
            // for more than immediate mutation or acquiring information.
            unsafe { &mut *x.get() }.push((self.flag(), self.data().get() as *mut _, self.size));
        });

        // unsafe: `item` is an `Rc`, which guarantees the existence and validity of the
        // pointee. It is also safeguarded by `self.used`, which we have proven above to be
        // `false`, otherwise we would have panicked.
        let object = unsafe { &mut *self.data().get() };
        let data = (handler)(object);

        STACK.with(|x| {
            // unsafe: We know there exist no other borrows of `STACK`. It is _never_ borrowed
            // for more than immediate mutation or acquiring information.
            let top = unsafe { &mut *x.get() }.pop();
            debug_assert!(top.is_some());
        });
        unborrow_mut(self.flag());
        data
    }

    /// Returns true if two `Node`s point to the same allocation.
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        Rc::ptr_eq(&this.item, &other.item)
    }

    fn data(&self) -> &UnsafeCell<T> {
        &self.item.1
    }

    fn flag(&self) -> &Cell<BorrowFlag> {
        &self.item.0
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn emit_works() {
        let node = Node::new(123);
        node.emit(|x| {
            assert_eq!(*x, 123);
            *x = 1;
        });
        node.emit(|x| {
            assert_eq!(*x, 1);
        });
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
        let node = Node::new_with_trace((), move |indent| {
            *capture.borrow_mut() = Some(indent);
        });

        node.emit(|_| {});

        assert!(matches!(*out.borrow(), Some(0)));
    }

    #[quickcheck_macros::quickcheck]
    fn tracing_nested(nestings: u8) {
        let out = Rc::new(RefCell::new(None));

        let capture = out.clone();
        let node = Node::new_with_trace((), move |indent| {
            *capture.borrow_mut() = Some(indent);
        });

        fn call(node: &Node<()>, count: u8) {
            if count == 0 {
                return;
            }
            node.emit(|x| {
                x.suspend(|| {
                    call(node, count - 1);
                });
            });
        };

        call(&node, nestings);

        if nestings == 0 {
            assert!(matches!(*out.borrow(), None));
        } else {
            assert_eq!(out.borrow().unwrap(), usize::from(nestings - 1));
        }
    }
}
