//! Synchronous publisher/subscriber broker event system for Rust.
//!
//! # Introduction #
//!
//! The synchronous publisher/subscriebr broker event system (event hub for short) is a mechanism by which objects can communicate without knowing about each
//! other. Event hubs are implemented by defining a set of signals to which an arbitrary struct
//! can subscribe. Said struct will then have to impl that signal and will be invoked once
//! that signal is called.
//!
//! # Example #
//!
//! ```
//! use revent::{hub, Hubbed, Selfscribe, Subscriber};
//!
//! // Main macro of this crate. Sets up types and boilerplate to make the event hub work.
//! hub! {
//!     // Module for holding helper structs and other things, a user does not need to
//!     // care about its contents.
//!     mod x;
//!     // Hub and Sub. The Hub is the top-level signal emitter; from the `Hub` you emit
//!     // signals. The `Sub` is similar to the `Hub` but allows one to subscribe to
//!     // signals, and is used only in the `Selfscribe` trait.
//!     Hub(Sub) {
//!         // A signal is just a function with the ability to
//!         // propagate further signals to `Subsignals`. You can decide which names
//!         // to use here.
//!         signal: fn(i32) -> (), Subsignals {
//!             // This is a "subsignal" - a signal which can be signalled from `signal`.
//!             subsignal: fn(str) -> (), EmptySignals,
//!         },
//!         subsignal: fn(str) -> (), EmptySignals {},
//!         unrelated: fn(()) -> (), UnrelatedSignals {},
//!     }
//! }
//!
//! // Create a hub
//! let hub = Hub::default();
//!
//! struct X;
//!
//! // Handler for the signal `signal`. Types in this impl must match the one given in the hub
//! // macro.
//! impl Subscriber<i32, (), Subsignals> for X {
//!     fn event(&mut self, input: &i32, subsignals: &Subsignals) {
//!         println!("Hello world!");
//!
//!         // We can now call signals defined in `Subsignals`.
//!         subsignals.subsignal("A string from signal");
//!     }
//! }
//!
//! impl Selfscribe<Sub> for X {
//!     fn subscribe(&mut self, this: &Hubbed<Self>, sub: &mut Sub) {
//!         // Make `X` subscribe to the signal `signal`.
//!         hub.signal(this);
//!     }
//! }
//!
//! // Add an object of X to this hub.
//! hub.subscribe(X);
//!
//! // Invoke `signal` on this hub.
//! hub.signal(&123);
//!
//! // Return values can be iterated from signals. Any unconsumed iterator values are consumed in
//! // the `Drop` implementation for this iterator.
//! for _ in hub.signal(&123) {
//!     // ...
//! }
//! ```
//!
//! # Validation #
//!
//! To avoid chained signalling (where one object signals itself causing mutable aliasing), or
//! recursive signalling (where one object signals another which signals back again) we perform
//! two checks at initialization time.
//!
//! Check 1 is performed during hub construction via `Hub::default()` and ensures that there
//! exists no possibility for subsignals to call into their originating signal.
//!
//! Check 2 is performed after a [Selfscribe] has performed a [Selfscribe::subscribe] and
//! ensures that the object does not have the ability to signal itself.
//!
//! Both checks ensure that no object gets mutably borrowed more than once.
//!
//! # Limitations #
//!
//! It is currently not possible to create a signal that has an argument containing lifetimes.
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

mod rec;

pub use rec::Recursion;

/// Wrapper around a shared pointer to a subscriber.
///
/// Allows `revent` to change its implementation without affecting its interface.
#[derive(Clone)]
pub struct Hubbed<T: ?Sized>(pub Rc<RefCell<T>>);

/// Subscriber trait for a specific input type and re-sending scheme.
///
/// Each subscriber gets an input `I` and needs to produce an output `O`. The subscriber also gets
/// a `signals` variable of type `&S`, allowing the subscriber to send further signals. `S` only
/// contains signals that are guaranteed to not call the current subscriber recursively.
pub trait Subscriber<I: ?Sized, O: Sized, S> {
    /// Handle an event.
    fn event(&mut self, input: &I, signals: &S) -> O;
}

/// `Selfscribe` allows a subscriber to decide which notification channels it will subscribe to.
pub trait Selfscribe<T>
where
    Self: Sized,
{
    /// Subscribe to various signals inside `hub`.
    fn subscribe(&mut self, this: &Hubbed<Self>, hub: &mut T);
}

/// Iterator over all subscriber return values. Will finish itself on `drop`.
///
/// Its internals are public but hidden in documentation. Please do not rely on the internals of
/// this iterator as they may change.
pub struct EmitIter<'a, 'b: 'a, I: ?Sized, O: Sized, S> {
    #[doc(hidden)]
    pub index: usize,
    #[doc(hidden)]
    pub input: &'b I,
    #[doc(hidden)]
    pub notify: S,
    #[doc(hidden)]
    pub topic: RefMut<'a, Vec<Rc<RefCell<dyn Subscriber<I, O, S>>>>>,
}

impl<'a, 'b: 'a, I: ?Sized, O: Sized, S> Iterator for EmitIter<'a, 'b, I, O, S> {
    type Item = O;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(subscriber) = self.topic.get(self.index) {
            self.index += 1;
            return Some(subscriber.borrow_mut().event(&self.input, &self.notify));
        }
        None
    }
}

impl<'a, 'b: 'a, I: ?Sized, O: Sized, S> Drop for EmitIter<'a, 'b, I, O, S> {
    fn drop(&mut self) {
        while let Some(_) = self.next() {}
    }
}

/// Generate an event hub.
#[macro_export]
macro_rules! hub {
    (mod $module:ident; $name:ident($sub:ident) { $($topic:ident: fn($input:ty) -> $output:ty, $notify:ident { $($channel:ident: fn($c_input:ty) -> $c_output:ty, $c_notify:ident),*$(,)? }),*$(,)? }) => {
        mod $module {
            use std::{cell::RefCell, rc::Rc};

            struct Sublist {
                $(pub $topic: Rc<RefCell<Vec<Rc<RefCell<dyn $crate::Subscriber<$input, $output, $notify>>>>>>),*,
            }

            impl Sublist {
                fn new() -> Self {
                    Self {
                        $($topic: Rc::new(RefCell::new(Vec::new()))),*,
                    }
                }
            }

            /// Invokes the basic signal.
            pub struct Invoker {
                base: Rc<Sublist>,
                recursion: $crate::Recursion,
            }

            impl Invoker {
                /// Create a new invoker.
                pub fn new(recursion: $crate::Recursion) -> Self {
                    Self {
                        base: Rc::new(Sublist::new()),
                        recursion,
                    }
                }

                $(
                    /// Invoke a signal.
                    pub fn $topic<'a, 'b>(&'a self, input: &'b $input, as_rc: &Rc<Self>) -> $crate::EmitIter<'a, 'b, $input, $output, $notify> {
                        let notify = $notify { invoker: as_rc.clone() };
                        $crate::EmitIter {
                            index: 0,
                            input,
                            notify,
                            topic: self.base.$topic.borrow_mut(),
                        }
                    }
                )*
            }

            $(
                /// Notification limiter. A slot can only send signals which do not cause recursion.
                /// This class only exposes signals to guarantee this.
                pub struct $notify {
                    #[allow(dead_code)]
                    invoker: Rc<Invoker>,
                }

                impl $notify {
                    $(
                        /// Emit a signal.
                        pub fn $channel<'a, 'b>(&'a self, input: &'b $c_input) -> $crate::EmitIter<'a, 'b, $c_input, $c_output, $c_notify> {
                            self.invoker.$channel(input, &self.invoker)
                        }
                    )*
                }
            )*

            /// Subscriber struct. Used inside `subscribe`.
            pub struct $sub {
                chain: Vec<&'static str>,
                invoker: Rc<Invoker>,
            }

            impl $sub {
                $(
                    /// Subscribe to a signal.
                    pub fn $topic<T: 'static + $crate::Subscriber<$input, $output, $notify>>(&mut self, this: &$crate::Hubbed<T>) {
                        self.chain.push(stringify!($topic));
                        self.invoker.base.$topic.borrow_mut().push(this.0.clone());
                    }
                )*
            }

            /// Hub of signals containing subscribers.
            pub struct $name {
                invoker: Rc<Invoker>,
            }

            impl $name {
                /// Add a subscriber to the hub.
                pub fn subscribe<T: $crate::Selfscribe<$sub>>(&self, subscriber: T) -> $crate::Hubbed<T> {
                    let mut sub = $sub {
                        chain: Vec::new(),
                        invoker: self.invoker.clone(),
                    };
                    let this = $crate::Hubbed(Rc::new(RefCell::new(subscriber)));
                    this.0.clone().borrow_mut().subscribe(&this, &mut sub);
                    self.invoker.recursion.is_chained(&sub.chain[..]).unwrap();
                    this
                }

                $(
                    /// Emit a signal.
                    pub fn $topic<'a, 'b>(&'a self, input: &'b $input) -> $crate::EmitIter<'a, 'b, $input, $output, $notify> {
                        self.invoker.$topic(input, &self.invoker)
                    }
                )*
            }

            impl Default for $name {
                fn default() -> Self {
                    let mut recursion = $crate::Recursion::default();
                    $(
                        recursion.add(stringify!($topic), &[$(stringify!($channel)),*]);
                    )*
                    recursion.check().unwrap();

                    Self {
                        invoker: Rc::new(Invoker::new(recursion)),
                    }
                }
            }
        }

        pub use $module::{$name, $sub, $($notify),*};
    };
}

#[cfg(test)]
mod tests {
    use crate::hub;

    #[test]
    fn basic() {
        hub! {
            mod x;
            Hub(Sub) {
                x: fn(()) -> (), X {},
            }
        }

        let hub = Hub::default();
        hub.x(&());
    }

    #[test]
    #[should_panic]
    fn recursive() {
        hub! {
            mod x;
            Hub(Sub) {
                x: fn(()) -> (), X {
                    y: fn(()) -> (), Y,
                },
                y: fn(()) -> (), Y {
                    x: fn(()) -> (), X,
                },
            }
        }

        let hub = Hub::default();
    }
}
