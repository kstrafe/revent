//! Synchronous event system.
//!
//! # What is an event system #
//!
//! An event system is a set of signals connected to a bunch of objects. When a signal is emitted,
//! the objects subscribing to said signal will have their handlers invoked to perform some useful
//! processing.
//!
//! ## Synchronous? ##
//!
//! Revent is synchonous, meaning that calling `emit` will immediately call all subscribers. This
//! also means that subscribers can return complex types with lifetimes referring to themselves.
//!
//! # Example #
//!
//! ```
//! use revent::{shared, Topic};
//!
//! // Create a trait for your event handler.
//! trait A {
//!     // Here is one "event". We call this on all handlers (implementors of A).
//!     fn a(&mut self);
//! }
//!
//! // Create a struct implementing A.
//! struct X;
//!
//! impl A for X {
//!     fn a(&mut self) {
//!         println!("Hello world");
//!     }
//! }
//!
//! // Create a new topic channel.
//! let mut topic: Topic<dyn A> = Topic::new();
//!
//! // Insert a new object of X.
//! topic.insert(shared(X));
//!
//! // Iterate over all objects and call the event function.
//! topic.emit(|x| {
//!     x.a();
//! });
//!
//! // Remove all subscribers to this topic.
//! topic.remove(|_| true);
//! ```
//!
//! # Nested emitting #
//!
//! To allow for nested emitting we simply provide a `&mut Topic<_>` to an event handler. The next
//! example shows this and its possible downside.
//!
//! # Mutable borrowing #
//!
//! It's possible to put a single object in two or more [Topic]s. If one topic is able to emit
//! into another topic then we may get a double-mutable borrow.
//!
//! The following code panics because of 2 mutable borrows.
//!
//! ```should_panic
//! use revent::{shared, Topic};
//!
//! trait A {
//!     fn a(&mut self, b: &mut Topic<dyn B>);
//! }
//!
//! trait B {
//!     fn b(&mut self);
//! }
//!
//! struct X;
//!
//! impl A for X {
//!     fn a(&mut self, b: &mut Topic<dyn B>) {
//!         b.emit(|x| {
//!             x.b();
//!         });
//!     }
//! }
//!
//! impl B for X {
//!     fn b(&mut self) {
//!     }
//! }
//!
//! let mut a: Topic<dyn A> = Topic::new();
//! let mut b: Topic<dyn B> = Topic::new();
//!
//! let x = shared(X);
//! a.insert(x.clone());
//! b.insert(x.clone());
//!
//! a.emit(|x| {
//!     x.a(&mut b);
//! });
//! ```
//!
//! To avoid this, design all your event handler types to never emit into a handler they subscribe
//! to.
//! In the above case we have `X` subscribe to `B` while also emitting into a `Topic<dyn B>`. For
//! larger systems it is highly advised to ensure all your event handlers never emit into a topic
//! they also subscribe to.
#![deny(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]
#![feature(drain_filter)]

use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
};

/// A topic to which objects can subscribe.
pub struct Topic<T: ?Sized>(Vec<Rc<RefCell<T>>>);

impl<T: ?Sized> Topic<T> {
    /// Create a new topic object.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a new subscriber into this topic.
    ///
    /// Appends the subscriber to the end of the subscriber list.
    pub fn insert(&mut self, item: Rc<RefCell<T>>) {
        self.access().push(item);
    }

    /// Modify all subscribers using the given closure.
    ///
    /// Iterates over all subscribers in subscription order.
    pub fn emit<F>(&mut self, mut caller: F)
    where
        F: FnMut(&mut T),
    {
        for item in self.access() {
            caller(&mut *revent_borrow_mut(item))
        }
    }

    /// Remove subscribers from this topic.
    ///
    /// Preserves the relative order of subscribers.
    /// Returns an iterator over all removed items.
    pub fn remove<'a, F>(&'a mut self, mut caller: F) -> impl Iterator<Item = Rc<RefCell<T>>> + 'a
    where
        F: FnMut(&mut T) -> bool + 'a,
    {
        self.access()
            .drain_filter(move |x| caller(&mut *revent_borrow_mut(x)))
    }

    fn access(&mut self) -> &mut Vec<Rc<RefCell<T>>> {
        &mut self.0
    }
}

impl<T: ?Sized> Default for Topic<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

/// Wrapper for `Rc::new(RefCell::new(_))`.
pub fn shared<T>(item: T) -> Rc<RefCell<T>> {
    Rc::new(RefCell::new(item))
}

fn revent_borrow_mut<T: ?Sized>(item: &RefCell<T>) -> RefMut<T> {
    match item.try_borrow_mut() {
        Ok(item) => item,
        Err(err) => {
            panic!("revent: {}", err);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn basic() {
        let mut topic: Topic<i32> = Topic::default();

        for i in 0..10 {
            topic.insert(shared(i));
        }

        let mut count = 0;
        topic.emit(|x| {
            assert_eq!(count, *x);
            count += 1;
        });
    }

    #[test]
    fn removal_yields_items() {
        let mut topic: Topic<i32> = Topic::default();

        for i in 0..10 {
            topic.insert(shared(i));
        }

        let removed = topic.remove(|x| *x > 7).count();
        assert_eq!(removed, 2);
    }

    #[test]
    fn trait_objects() {
        pub trait A {}

        let mut x: Topic<dyn A> = Default::default();

        struct X;
        impl A for X {}

        x.insert(shared(X));

        let mut count = 0;
        x.emit(|_| {
            count += 1;
        });
        assert_eq!(count, 1);
    }

    #[test]
    #[should_panic(expected = "revent: already borrowed")]
    fn double_mutable_borrow() {
        trait A {
            fn a(&mut self, b: &mut Topic<dyn B>);
        }

        trait B {
            fn b(&mut self);
        }

        struct X;

        impl A for X {
            fn a(&mut self, b: &mut Topic<dyn B>) {
                b.emit(|x| {
                    x.b();
                });
            }
        }

        impl B for X {
            fn b(&mut self) {}
        }

        let mut a: Topic<dyn A> = Topic::new();
        let mut b: Topic<dyn B> = Topic::new();

        let x = shared(X);
        a.insert(x.clone());
        b.insert(x.clone());

        a.emit(|x| {
            x.a(&mut b);
        });
    }
}
