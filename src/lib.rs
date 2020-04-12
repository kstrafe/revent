//! Synchronous, recursive event system.
//!
//! # Introduction #
//!
//! An event system is a collection of objects that can receive and send signals.
//!
//! In `revent` we construct something called a `hub` which contains [Channel]s and/or [Slot]s.
//! Each such signal container is just a list of [Node]s (objects) interested in said signal.
//!
//! Emitting an event is as simple as just running a function on the relevant channel.
//!
//! `revent`'s types ensure that we do _not_ need [RefCell](std::cell::RefCell), and that we do not
//! accidentally mutably alias. Recursion can be achieved by suspending access to an object's `&mut
//! Self`.
//!
//! # Example #
//!
//! ```
//! use revent::{Channel, Node, Slot, Suspend};
//!
//! // Create signal traits.
//! trait SignalA { fn signal_a(&mut self, hub: &MyHub); }
//! trait SignalB { fn signal_b(&mut self, hub: &MyHub); }
//! trait SignalC { fn signal_c(&mut self); }
//!
//! // Create a struct of channels and slots based on your signal traits.
//! #[derive(Default)]
//! struct MyHub {
//!     signal_a: Channel<dyn SignalA>,
//!     signal_b: Channel<dyn SignalB>, // A channel contains any number of nodes.
//!     signal_c: Slot<dyn SignalC>, // A slot contains only a single node.
//! }
//!
//! // Create trait implementors. Note that `A` implements both `SignalA` and `SignalB`.
//! struct A; struct B; struct C;
//!
//! impl SignalA for A {
//!     fn signal_a(&mut self, hub: &MyHub) {
//!         println!("A::signal_a: {:?}", self as *mut _);
//!
//!         self.suspend(|| { // Suspend here in order to not panic. `signal_b` also contains this
//!             hub.signal_b.emit(|x| { // object, so we must ensure we relinquish access to `&mut`.
//!                 x.signal_b(hub);
//!             });
//!         });
//!     }
//! }
//! impl SignalB for A {
//!     fn signal_b(&mut self, _: &MyHub) {
//!         println!("A::signal_b: {:?}", self as *mut _);
//!     }
//! }
//! impl SignalB for B {
//!     fn signal_b(&mut self, hub: &MyHub) {
//!         println!("B::signal_b: {:?}", self as *mut _);
//!         hub.signal_c.emit(|x| { // We can also emit without suspending self. If the channel or
//!         // slot we emit into contains the object from which we emit, then a panic will occur.
//!             x.signal_c();
//!         });
//!     }
//! }
//! impl SignalC for C {
//!     fn signal_c(&mut self) {
//!         println!("C::signal_c: {:?}", self as *mut _);
//!     }
//! }
//!
//! // Instantiate `MyHub`.
//! let mut hub = MyHub::default();
//!
//! // Insert nodes into the hub. Nodes can be cloned and used on their own using the `emit`
//! // method.
//! let a = Node::new(A);
//! hub.signal_a.insert(a.clone());
//! hub.signal_b.insert(a.clone());
//! hub.signal_b.insert(Node::new(B));
//! hub.signal_c.insert(Node::new(C));
//!
//! // Run `a` and call `signal_a`.
//! a.emit(|x| {
//!     x.signal_a(&hub);
//! });
//! ```
//!
//! Output:
//!
//! ```ignore
//! A::signal_a: 0x55efd14a8b70
//! A::signal_b: 0x55efd14a8b70
//! B::signal_b: 0x55efd14a8bf0
//! C::signal_c: 0x55efd14a8c50
//! ```
#![deny(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
#![feature(coerce_unsized, unsize)]

pub use self::{channel::Channel, node::Node, slot::Slot};
use std::cell::{Cell, UnsafeCell};

mod channel;
mod node;
mod slot;

// ---

#[inline(always)]
fn borrow(value: &Cell<BorrowFlag>) {
    value.set(value.get() + 1);
}

#[inline(always)]
fn unborrow(value: &Cell<BorrowFlag>) {
    value.set(value.get() - 1);
}

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

#[inline(always)]
fn is_borrowed_mut(value: &Cell<BorrowFlag>) -> bool {
    value.get() < 0
}

type BorrowFlag = isize;

// ---

thread_local! {
    // `STACK` is parallel to the callstack. The last element represents the current active item
    // being invoked on a `Channel` or `Slot`. It is inside an `UnsafeCell` because it is only ever
    // pushed/popped in the same function, and we can prove that borrows are not propagated.
    static STACK: UnsafeCell<Vec<(*const Cell<BorrowFlag>, *mut ())>> = UnsafeCell::new(Vec::new());
}

// ---

/// Suspend an arbitrary `&mut` from access.
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
    fn suspend<F: FnOnce() -> R, R>(&mut self, runner: F) -> R {
        let last = STACK.with(|x| {
            // unsafe: We know there exist no other borrows of `STACK`. It is _never_ borrowed
            // for more than immediate mutation or acquiring information.
            if let Some(last) = unsafe { &mut *x.get() }.last() {
                *last
            } else {
                panic!("revent: suspend: item not expected");
            }
        });

        let item: *mut _ = self;
        if last.1 != item as *mut () {
            panic!("revent: suspend: item not expected",);
        }

        // unsafe: The pointer `last.0` to `*const Cell<bool>` is valid because it refers to a
        // variable on the stack from at least 2 stack frames earlier. The pointer comes from
        // `Node` which is contained by `Channel` or `Slot`, which guarantees that the pointee
        // exists.
        //
        // We do _not_ need to check the value of the borrow flag since we got `&mut`, so we know
        // it is guaranteed a mutable borrow.
        unborrow_mut(unsafe { &*last.0 });
        let data = (runner)();
        // unsafe: See above.
        borrow_mut(unsafe { &*last.0 });
        data
    }

    /// Same as [suspend](Suspend::suspend) but takes an immutable reference instead.
    fn suspend_ref<F: FnOnce() -> R, R>(&self, runner: F) -> R {
        let last = STACK.with(|x| {
            // unsafe: We know there exist no other borrows of `STACK`. It is _never_ borrowed
            // for more than immediate mutation or acquiring information.
            if let Some(last) = unsafe { &mut *x.get() }.last() {
                *last
            } else {
                panic!("revent: suspend_ref: item not expected");
            }
        });

        let item: *const _ = &*self;
        let raw: *const _ = last.1;
        if raw != item as *const () {
            panic!("revent: suspend_ref: item not expected");
        }

        // We _must_ guarantee that this an immutable borrow (corresponding to an `emit_ref`). If
        // it is not, then our borrow flag won't make sense anymore.
        if is_borrowed_mut(unsafe { &*last.0 }) {
            panic!("revent: suspend_ref: called on item that is mutably borrowed");
        }

        // unsafe: The pointer `last.0` to `*const Cell<bool>` is valid because it refers to a
        // variable on the stack from at least 2 stack frames earlier. The pointer comes from
        // `Node` which is contained by `Channel` or `Slot`, which guarantees that the pointee
        // exists.
        unborrow(unsafe { &*last.0 });
        let data = (runner)();
        // unsafe: See above.
        borrow(unsafe { &*last.0 });
        data
    }
}

impl<T> Suspend for T {}

// ---

#[cfg(test)]
mod tests {
    use crate::*;
    use std::cell::Cell;

    #[test]
    #[should_panic(expected = "revent: suspend: item not expected")]
    fn suspending_on_empty_stack() {
        ().suspend(|| {});
    }

    #[test]
    #[should_panic(expected = "revent: suspend_ref: item not expected")]
    fn suspending_ref_on_empty_stack() {
        ().suspend_ref(|| {});
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
    #[should_panic(expected = "revent: suspend_ref: item not expected")]
    fn suspending_ref_invalid_item() {
        let x = Node::new(());
        x.emit(|()| {
            ().suspend_ref(|| {});
        });
    }

    #[test]
    #[should_panic(expected = "revent: suspend_ref: called on item that is mutably borrowed")]
    fn suspending_ref_of_emit() {
        let x = Node::new(());
        x.emit(|x| {
            (&*x).suspend_ref(|| {});
        });
    }

    #[test]
    #[should_panic(expected = "revent: emit_ref: accessing already mutably borrowed item")]
    fn emit_ref_after_emit() {
        let x = Node::new(());
        x.emit(|()| {
            x.emit_ref(|()| {});
        });
    }

    #[test]
    #[should_panic(expected = "revent: emit: accessing already borrowed item")]
    fn emit_after_emit_ref() {
        let x = Node::new(());
        x.emit_ref(|()| {
            x.emit(|()| {});
        });
    }

    #[test]
    fn ref_suspend_followed_by_suspend() {
        let node = Node::new(());
        node.emit_ref(|x| {
            x.suspend_ref(|| {
                node.emit(|x| {
                    x.suspend(|| {});
                });
            });
        });
    }

    #[test]
    fn recursion() {
        trait Trait1 {
            fn channel_1_function(&mut self, hub: &Hub);
        }

        struct Hub {
            channel1: Channel<dyn Trait1>,
        }

        let mut hub = Hub {
            channel1: Channel::new(),
        };

        struct My {
            value: usize,
        };

        impl Trait1 for My {
            fn channel_1_function(&mut self, hub: &Hub) {
                if self.value == 0 {
                    return;
                }

                self.value -= 1;

                self.suspend(|| {
                    hub.channel1.emit(|item| {
                        item.channel_1_function(hub);
                    });
                });
            }
        }

        hub.channel1.insert(Node::new(My { value: 12 }));

        hub.channel1.emit(|x| {
            x.channel_1_function(&hub);
        });
    }

    #[test]
    fn recursion_ref() {
        trait Trait {
            fn function(&self, channel: &Channel<dyn Trait>);
        }

        let mut channel = Channel::<dyn Trait>::new();

        struct My {
            value: Cell<usize>,
        };

        impl Trait for My {
            fn function(&self, channel: &Channel<dyn Trait>) {
                if self.value.get() == 0 {
                    return;
                }

                self.value.set(self.value.get() - 1);

                self.suspend_ref(|| {
                    channel.emit_ref(|item| {
                        item.function(channel);
                    });
                });
            }
        }

        channel.insert(Node::new(My {
            value: Cell::new(12),
        }));

        channel.emit_ref(|x| {
            x.function(&channel);
        });
    }
}
