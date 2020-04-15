//! Synchronous and recursive event system.
//!
//! # Introduction #
//!
//! An event system is a collection of objects that can receive and send signals.
//!
//! In `revent` we construct something called a [Node] containing an object of interest. This
//! object can invoke objects in other `node`s, which in turn can invoke other objects - including
//! the originator - safely. This is done by [suspend](crate::Suspend::suspend)ing the `&mut self` to the contents of a node.
//!
//! # Example #
//!
//! ```
//! use revent::Node;
//!
//! let number = Node::new(123);
//!
//! number.emit(|n| {
//!     println!("{}", *n);
//!     *n = 100;
//!     println!("{}", *n);
//! });
//! ```
//!
//! See the documentation for [Channel] and [Slot] and [Suspend] for examples.
//!
//! # Intent #
//!
//! This library is intended to be used as a "suspendable `RefCell`" by a bunch of objects which
//! wish to communicate with each other without using a central mediator object.
//!
//! For instance, one can create a `revent::Channel<dyn MyTrait>` which contains objects of
//! interest, where each object can inside its own handler do something along the lines of:
//! ```
//! use revent::{Channel, Suspend};
//! trait MyTrait {
//!     fn function(&mut self, channel: &Channel<dyn MyTrait>);
//! }
//!
//! struct MyObject;
//! impl MyTrait for MyObject {
//!     fn function(&mut self, channel: &Channel<dyn MyTrait>) {
//!         // Do something...
//!         self.suspend(|| {
//!             channel.emit(|x| {
//!                 x.function(channel);
//!             });
//!         });
//!         // Do something else...
//!     }
//! }
//! ```
//!
//! The above allows the object to emit a signal on a channel it is part of, even calling itself
//! recursively without mutably aliasing by suspending `&mut self`.
#![deny(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
#![feature(coerce_unsized, unsize)]

use self::trace::Trace;
pub use self::{channel::Channel, node::Node, slot::Slot};
use std::{
    cell::{Cell, UnsafeCell},
    mem,
};

mod channel;
mod node;
mod slot;
mod trace;

// ---

#[inline(always)]
fn borrow_mut(value: &Cell<BorrowFlag>) {
    value.set(value.get() - 1);
}

#[inline(always)]
fn unborrow_mut(value: &Cell<BorrowFlag>) {
    value.set(value.get() + 1);
}

#[inline(always)]
fn is_borrowed(value: &Cell<BorrowFlag>) -> bool {
    value.get() != 0
}

type BorrowFlag = isize;

// ---

thread_local! {
    // `STACK` is parallel to the callstack. The last element represents the current active item
    // being invoked on a `Node`. It is inside an `UnsafeCell` because it is only ever
    // pushed/popped in the same function, and we can prove that borrows are not propagated.
    static STACK: UnsafeCell<Vec<(*const Cell<BorrowFlag>, *mut (), usize)>> = UnsafeCell::new(Vec::new());
}

// ---

/// Suspend an arbitrary reference from access.
pub trait Suspend {
    /// Suspend this object and run `runner`, which by using another data structure can reborrow
    /// `&mut Self` without violating the mutable aliasing rules.
    ///
    /// Only the last emitted object can be suspended.
    ///
    /// # Panics #
    ///
    /// Panics if the suspended object is not stored in a [Node], or if the object is not at the
    /// top of the current node stack.
    ///
    /// ```should_panic
    /// use revent::{Node, Suspend};
    /// let node1 = Node::new(());
    /// let node2 = Node::new(());
    /// node1.emit(|x1| {
    ///     node2.emit(|x2| {
    ///         x1.suspend(|| {}); // Panic, `node2` was last emitted, we can't suspend `x1`, which
    ///         // comes from `node1`.
    ///     });
    /// });
    /// ```
    fn suspend<F: FnOnce() -> R, R>(&mut self, runner: F) -> R
    where
        Self: Sized,
    {
        let last = STACK.with(|x| {
            // unsafe: We know there exist no other borrows of `STACK`. It is _never_ borrowed
            // for more than immediate mutation or acquiring information.
            if let Some(last) = unsafe { &mut *x.get() }.last() {
                *last
            } else {
                panic!("revent: suspend: not inside node context");
            }
        });

        let item: *mut _ = self;
        if last.1 != item as *mut () || last.2 != mem::size_of::<Self>() {
            panic!("revent: suspend: item not expected",);
        }

        // unsafe: The pointer `last.0` to `*const Cell<BorrowFlag>` is valid because it refers to a
        // variable on the stack from at least 2 stack frames earlier. The pointer comes from
        // `Node` which guarantees that the pointee exists.
        //
        // We do _not_ need to check the value of the borrow flag since we got `&mut`, so we know
        // it is guaranteed a mutable borrow.
        unborrow_mut(unsafe { &*last.0 });
        let data = (runner)();
        // unsafe: See above.
        borrow_mut(unsafe { &*last.0 });
        data
    }
}

impl<T> Suspend for T {}

// ---

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    #[should_panic(expected = "revent: suspend: not inside node context")]
    fn suspending_on_empty_stack() {
        ().suspend(|| {});
    }

    #[test]
    #[should_panic(expected = "revent: suspend: item not expected")]
    fn suspending_invalid_item() {
        let x = Node::new(());
        x.emit(|()| {
            ().suspend(|| {});
        });
    }

    #[test]
    #[should_panic(expected = "revent: suspend: item not expected")]
    fn suspend_not_top() {
        let x = Node::new(());
        let y = Node::new(());
        x.emit(|x1| {
            y.emit(|_| {
                x1.suspend(|| {});
            });
        });
    }

    #[test]
    fn suspend() {
        let x = Node::new(());
        x.emit(|y| {
            y.suspend(|| {});
        });
    }

    #[test]
    fn recursion() {
        trait Trait {
            fn function(&mut self, hub: &Channel<dyn Trait>);
        }

        let mut channel = Channel::<dyn Trait>::new();

        struct My {
            value: usize,
        };

        impl Trait for My {
            fn function(&mut self, channel: &Channel<dyn Trait>) {
                if self.value == 0 {
                    return;
                }

                self.value -= 1;

                self.suspend(|| {
                    channel.emit(|item| {
                        item.function(channel);
                    });
                });
            }
        }

        channel.insert(0, Node::new(My { value: 12 }));

        channel.emit(|x| {
            x.function(&channel);
        });
    }

    #[test]
    #[should_panic(expected = "revent: suspend: item not expected")]
    fn suspend_overlapping_struct_check() {
        struct Decoy {
            a: (),
            _b: u8,
        }

        let my_node = Node::new(Decoy { a: (), _b: 0 });

        my_node.emit(|x| {
            x.a.suspend(|| {});
        });
    }

    #[test]
    #[should_panic(expected = "revent: emit: accessing already borrowed item")]
    fn node_inside_node() {
        let node = Node::new(Node::new(123));
        node.emit(|subnode| {
            subnode.emit(|x| {
                x.suspend(|| {
                    node.emit(|subnode| {
                        *subnode = Node::new(100);
                    });
                });
            });
        });
    }
}
