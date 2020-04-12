use crate::{
    borrow, borrow_mut, is_borrowed, is_borrowed_mut, unborrow, unborrow_mut, BorrowFlag, STACK,
};
use std::{
    cell::{Cell, UnsafeCell},
    marker::Unsize,
    ops::CoerceUnsized,
    rc::Rc,
};

/// Node containing arbitrary data.
///
/// Ensures that no double-mutable borrows exist and allows the contained item to
/// [Suspend](crate::Suspend) itself.
///
/// Node is fundamentally the same as [RefCell](std::cell::RefCell), but does one more thing:
/// it allows suspension of the last emitted node by using `&mut` or `&`. Suspending allows the
/// node to be reborrowed without aliasing.
pub struct Node<T: ?Sized> {
    item: Rc<(Cell<BorrowFlag>, UnsafeCell<T>)>,
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
        }
    }
}

impl<T> Node<T> {
    /// Create a new node.
    pub fn new(item: T) -> Self {
        Self {
            item: Rc::new((Cell::new(0), UnsafeCell::new(item))),
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
        if is_borrowed(self.flag()) {
            panic!("revent: emit: accessing already borrowed item");
        }
        borrow_mut(self.flag());

        STACK.with(|x| {
            // unsafe: We know there exist no other borrows of `STACK`. It is _never_ borrowed
            // for more than immediate mutation or acquiring information.
            unsafe { &mut *x.get() }.push((self.flag(), self.data().get() as *mut _));
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

    /// Acquire a `&` to the contents of the node and allow it to [Suspend](crate::Suspend) itself.
    ///
    /// Immutable version of [emit](Node::emit).
    pub fn emit_ref<F: FnOnce(&T) -> R, R>(&self, handler: F) -> R {
        if is_borrowed_mut(self.flag()) {
            panic!("revent: emit_ref: accessing already mutably borrowed item");
        }
        borrow(self.flag());

        STACK.with(|x| {
            // unsafe: We know there exist no other borrows of `STACK`. It is _never_ borrowed
            // for more than immediate mutation or acquiring information.
            unsafe { &mut *x.get() }.push((self.flag(), self.data().get() as *mut _));
        });

        // unsafe: `item` is an `Rc`, which guarantees the existence and validity of the
        // pointee. It is also safeguarded by `self.used`, which we have proven above to be
        // `false`, otherwise we would have panicked.
        let object = unsafe { &*self.data().get() };
        let data = (handler)(object);

        STACK.with(|x| {
            // unsafe: We know there exist no other borrows of `STACK`. It is _never_ borrowed
            // for more than immediate mutation or acquiring information.
            let top = unsafe { &mut *x.get() }.pop();
            debug_assert!(top.is_some());
        });
        unborrow(self.flag());
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
