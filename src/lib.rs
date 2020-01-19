//! Event broker library for Rust.
//!
//! Implements a synchronous transitive event broker that does not violate mutability constraints.
//! It does so by performing DAG traversals to ensure that no signal chains are able
//! to form a loop.
//!
//! # What is an event broker? #
//!
//! An event broker is a bag of objects and a bunch of "signals". Each object decides which signals to listen to.
//!
//! Each object (also called subscriber) is notified when its subscribed signal is [emit](crate::Topic::emit)ted.
//! A subscriber may during notification processing emit yet another signal into the broker, and so
//! on.
//!
//! ```
//! use revent::{hub, Shared, Subscriber};
//!
//! // Construct a trait for some type of event.
//! trait MyEventHandler {
//!     fn my_event(&mut self);
//! }
//!
//! // Create an event hub.
//! hub! {
//!     Hub {
//!         event: dyn MyEventHandler,
//!     }
//! }
//!
//! // Construct a hub object.
//! let hub = Hub::default();
//!
//! // Implement a subscriber to some event.
//! struct X;
//! impl MyEventHandler for X {
//!     fn my_event(&mut self) {
//!         println!("Hello world");
//!     }
//! }
//! impl Subscriber<Hub> for X {
//!     type Input = ();
//!     fn build(_: Hub, input: Self::Input) -> Self {
//!         Self
//!     }
//!     fn subscribe(hub: &Hub, shared: Shared<Self>) {
//!         hub.event.subscribe(shared);
//!     }
//! }
//!
//! // Create an instance of X in the hub.
//! hub.subscribe::<X>(());
//!
//! // Now emit an event into the topic.
//! hub.event.emit(|x| {
//!     x.my_event();
//! });
//! ```
//!
//! # Logging #
//!
//! Use `feature = "slog"` to add a method `log` to the hub generated from the [hub] macro.
//! This method sets a logger object.
#![deny(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
#![feature(coerce_unsized, drain_filter, unsize)]

pub mod example;
mod mng;
mod shared;
mod topic;
pub use mng::Manager;
pub use shared::Shared;
pub use topic::Topic;

/// Generate an event hub and its associated boilerplate code.
///
/// ```
/// use revent::hub;
///
/// pub trait MyTrait1 {}
/// pub trait MyTrait2 {}
///
/// hub! {
///     MyHub {
///         channel_name1: dyn MyTrait1,
///         channel_name2: dyn MyTrait2,
///     }
/// }
///
/// let my_hub = MyHub::default();
/// // or
/// let my_hub = MyHub::new();
///
/// my_hub.channel_name1.emit(|_| {
///     // Do something with each subscriber of channel_name1.
/// });
/// ```
///
/// The macro generates a struct of `MyHub` containing all topics. [Topic]s are public members of
/// the struct. In addition, [Default] is implemented as well as `new` and `subscribe`.
#[macro_export]
macro_rules! hub {
    ($hub:ident { $($channel:ident: $type:ty),*$(,)? }) => {
        /// Hub of events.
        ///
        /// Contains various topics which can be emitted into or subscribed to.
        pub struct $hub {
            $(
                /// Channel for the given type of event handler.
                pub $channel: $crate::Topic<$type>
            ),*,
            // TODO: When gensyms are supported make this symbol a gensym.
            #[doc(hidden)]
            pub _manager: ::std::rc::Rc<::std::cell::RefCell<$crate::Manager>>,
        }

        impl Default for $hub {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $hub {
            /// Create a new hub.
            pub fn new() -> Self {
                let mng = ::std::rc::Rc::new(::std::cell::RefCell::new($crate::Manager::default()));
                Self {
                    $($channel: $crate::Topic::new(stringify!($channel), &mng)),*,
                    _manager: mng,
                }
            }

            /// Set a logger for this hub.
            #[allow(dead_code)]
            #[cfg(feature = "slog")]
            pub fn log(self, logger: slog::Logger) -> Self {
                self._manager.borrow_mut().log = logger;
                self
            }

            /// Insert a subscriber into the hub.
            pub fn subscribe<T: $crate::Subscriber<Self>>(&self, input: T::Input) {
                self.manager().borrow_mut().begin_construction();
                let hub = self.clone_deactivate();
                let shared = $crate::Shared::new(T::build(hub, input));
                T::subscribe(self, shared);
                self.manager().borrow_mut().end_construction();
            }

            /// Generate a graphviz (dot) style graph.
            #[allow(dead_code)]
            pub fn graph(&self) -> String {
                self.manager().borrow_mut().graphviz()
            }

            #[doc(hidden)]
            fn clone_deactivate(&self) -> Self {
                Self {
                    $($channel: self.$channel.clone_deactivate()),*,
                    _manager: self.manager().clone(),
                }
            }

            #[doc(hidden)]
            pub fn manager(&self) -> ::std::rc::Rc<::std::cell::RefCell<$crate::Manager>> {
                self._manager.clone()
            }
        }
    };
}

/// Subscriber to an event hub.
///
/// Is used by the `subscribe` function generated by the [hub](hub) macro.
pub trait Subscriber<T>
where
    Self: Sized,
{
    /// Input data to the build function.
    type Input;
    /// Build an object using any hub and arbitrary input.
    fn build(hub: T, input: Self::Input) -> Self;
    /// Subscribe to a specific hub.
    ///
    /// This function wraps the self object inside an opaque wrapper which can be used on
    /// [Topic::subscribe].
    fn subscribe(hub: &T, shared: Shared<Self>);
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::{cell::Cell, rc::Rc};

    #[test]
    fn simple_listener() {
        pub trait EventHandler {}

        hub! {
            Hub {
                event: dyn EventHandler,
            }
        }

        let hub = Hub::default();

        struct X;
        impl EventHandler for X {}
        impl Subscriber<Hub> for X {
            type Input = ();
            fn build(_: Hub, _: Self::Input) -> Self {
                Self
            }
            fn subscribe(hub: &Hub, shared: Shared<Self>) {
                hub.event.subscribe(shared);
            }
        }

        hub.subscribe::<X>(());

        let mut count = 0;
        hub.event.emit(|_| {
            count += 1;
        });
        assert_eq!(count, 1);
    }

    #[test]
    #[should_panic(expected = "Topic is not active (emit): event2")]
    fn emit_on_non_activated_channel() {
        pub trait EventHandler {
            fn event(&mut self);
        }

        hub! {
            Hub {
                event1: dyn EventHandler,
                event2: dyn EventHandler,
            }
        }

        let hub = Hub::default();

        struct X {
            hub: Hub,
        }
        impl EventHandler for X {
            fn event(&mut self) {
                self.hub.event2.emit(|_| {});
            }
        }
        impl Subscriber<Hub> for X {
            type Input = ();
            fn build(hub: Hub, _: Self::Input) -> Self {
                Self { hub }
            }
            fn subscribe(hub: &Hub, shared: Shared<Self>) {
                hub.event1.subscribe(shared);
            }
        }

        hub.subscribe::<X>(());

        hub.event1.emit(|x| {
            x.event();
        });
    }

    #[test]
    #[should_panic(expected = "Recursion detected: [\"event\"]")]
    fn recursion_to_self() {
        pub trait EventHandler {}

        hub! {
            Hub {
                event: dyn EventHandler,
            }
        }

        let hub = Hub::default();

        struct X;
        impl EventHandler for X {}
        impl Subscriber<Hub> for X {
            type Input = ();
            fn build(mut hub: Hub, _: Self::Input) -> Self {
                hub.event.activate();
                Self
            }
            fn subscribe(hub: &Hub, shared: Shared<Self>) {
                hub.event.subscribe(shared);
            }
        }

        hub.subscribe::<X>(());
    }

    #[test]
    #[should_panic(expected = "Recursion detected: [\"event1\", \"event2\"]")]
    fn transitive_recursion() {
        pub trait EventHandler {}

        hub! {
            Hub {
                event1: dyn EventHandler,
                event2: dyn EventHandler,
            }
        }

        let hub = Hub::default();

        struct X;
        impl EventHandler for X {}
        impl Subscriber<Hub> for X {
            type Input = ();
            fn build(mut hub: Hub, _: Self::Input) -> Self {
                hub.event1.activate();
                Self
            }
            fn subscribe(hub: &Hub, shared: Shared<Self>) {
                hub.event2.subscribe(shared);
            }
        }

        struct Y;
        impl EventHandler for Y {}
        impl Subscriber<Hub> for Y {
            type Input = ();
            fn build(mut hub: Hub, _: Self::Input) -> Self {
                hub.event2.activate();
                Self
            }
            fn subscribe(hub: &Hub, shared: Shared<Self>) {
                hub.event1.subscribe(shared);
            }
        }

        hub.subscribe::<X>(());
        hub.subscribe::<Y>(());
    }

    #[test]
    fn recursion_is_per_object() {
        pub trait EventHandler {
            fn event(&mut self);
        }

        hub! {
            Hub {
                event1: dyn EventHandler,
                event2: dyn EventHandler,
            }
        }

        let hub = Hub::default();

        struct X {
            hub: Hub,
            called: Rc<Cell<usize>>,
        }
        impl EventHandler for X {
            fn event(&mut self) {
                self.called.set(self.called.get() + 1);
                self.hub.event2.emit(|x| {
                    x.event();
                });
            }
        }
        impl Subscriber<Hub> for X {
            type Input = Rc<Cell<usize>>;
            fn build(mut hub: Hub, called: Self::Input) -> Self {
                hub.event2.activate();
                Self { hub, called }
            }
            fn subscribe(hub: &Hub, shared: Shared<Self>) {
                hub.event1.subscribe(shared);
            }
        }

        struct Y {
            called: Rc<Cell<usize>>,
        }
        impl EventHandler for Y {
            fn event(&mut self) {
                self.called.set(self.called.get() + 1);
            }
        }
        impl Subscriber<Hub> for Y {
            type Input = Rc<Cell<usize>>;
            fn build(_: Hub, called: Self::Input) -> Self {
                Self { called }
            }
            fn subscribe(hub: &Hub, shared: Shared<Self>) {
                hub.event1.subscribe(shared.clone());
                hub.event2.subscribe(shared);
            }
        }

        let called_x = Rc::new(Cell::new(0));
        let called_y = Rc::new(Cell::new(0));

        hub.subscribe::<X>(called_x.clone());
        hub.subscribe::<Y>(called_y.clone());

        assert_eq!(called_x.get(), 0);
        assert_eq!(called_y.get(), 0);

        hub.event1.emit(EventHandler::event);

        assert_eq!(called_x.get(), 1);
        assert_eq!(called_y.get(), 2);

        hub.event2.emit(EventHandler::event);

        assert_eq!(called_x.get(), 1);
        assert_eq!(called_y.get(), 3);
    }

    #[test]
    fn no_subscription_is_dropped() {
        pub trait EventHandler {}

        hub! {
            Hub {
                event: dyn EventHandler,
            }
        }

        let hub = Hub::default();

        struct X {
            dropped: Rc<Cell<bool>>,
        }
        impl EventHandler for X {}
        impl Subscriber<Hub> for X {
            type Input = Rc<Cell<bool>>;
            fn build(_: Hub, input: Self::Input) -> Self {
                Self { dropped: input }
            }
            fn subscribe(_: &Hub, _: Shared<Self>) {}
        }
        impl Drop for X {
            fn drop(&mut self) {
                self.dropped.set(true);
            }
        }

        let dropped: Rc<Cell<bool>> = Default::default();
        assert_eq!(dropped.get(), false);
        hub.subscribe::<X>(dropped.clone());
        assert_eq!(dropped.get(), true);
    }

    #[test]
    #[cfg(feature = "slog")]
    fn with_slog() {
        use slog::{Drain, Key, Serializer, KV};
        use std::{
            fmt::Arguments,
            io::Error,
            sync::{Arc, Mutex},
        };

        pub trait EventHandler {}

        hub! {
            Hub {
                event: dyn EventHandler,
            }
        }

        #[derive(Clone, Default)]
        struct Buffer {
            string: Arc<Mutex<String>>,
        }

        impl Drain for Buffer {
            type Ok = ();
            type Err = Error;
            fn log(
                &self,
                record: &slog::Record,
                values: &slog::OwnedKVList,
            ) -> Result<Self::Ok, Self::Err> {
                #[derive(Default)]
                struct Kvencode {
                    kvs: Vec<(String, String)>,
                }

                impl Kvencode {
                    fn add<T: ToString>(&mut self, key: Key, val: T) -> slog::Result {
                        self.kvs.push((key.to_string(), val.to_string()));
                        Ok(())
                    }

                    fn finish(self) -> String {
                        let mut kvs = String::new();
                        for (k, v) in self.kvs.iter().rev() {
                            kvs += &format!(", {}={}", k, v);
                        }
                        kvs
                    }
                }

                impl Serializer for Kvencode {
                    fn emit_arguments(&mut self, key: Key, val: &Arguments) -> slog::Result {
                        self.add(key, val)
                    }
                    fn emit_usize(&mut self, key: Key, val: usize) -> slog::Result {
                        self.add(key, val)
                    }
                    fn emit_isize(&mut self, key: Key, val: isize) -> slog::Result {
                        self.add(key, val)
                    }
                    fn emit_bool(&mut self, key: Key, val: bool) -> slog::Result {
                        self.add(key, val)
                    }
                    fn emit_char(&mut self, key: Key, val: char) -> slog::Result {
                        self.add(key, val)
                    }
                    fn emit_u8(&mut self, key: Key, val: u8) -> slog::Result {
                        self.add(key, val)
                    }
                    fn emit_i8(&mut self, key: Key, val: i8) -> slog::Result {
                        self.add(key, val)
                    }
                    fn emit_u16(&mut self, key: Key, val: u16) -> slog::Result {
                        self.add(key, val)
                    }
                    fn emit_i16(&mut self, key: Key, val: i16) -> slog::Result {
                        self.add(key, val)
                    }
                    fn emit_u32(&mut self, key: Key, val: u32) -> slog::Result {
                        self.add(key, val)
                    }
                    fn emit_i32(&mut self, key: Key, val: i32) -> slog::Result {
                        self.add(key, val)
                    }
                    fn emit_f32(&mut self, key: Key, val: f32) -> slog::Result {
                        self.add(key, val)
                    }
                    fn emit_f64(&mut self, key: Key, val: f64) -> slog::Result {
                        self.add(key, val)
                    }
                    fn emit_u64(&mut self, key: Key, val: u64) -> slog::Result {
                        self.add(key, val)
                    }
                    fn emit_i64(&mut self, key: Key, val: i64) -> slog::Result {
                        self.add(key, val)
                    }
                    fn emit_u128(&mut self, key: Key, val: u128) -> slog::Result {
                        self.add(key, val)
                    }
                    fn emit_i128(&mut self, key: Key, val: i128) -> slog::Result {
                        self.add(key, val)
                    }
                    fn emit_str(&mut self, key: Key, val: &str) -> slog::Result {
                        self.add(key, val)
                    }
                    fn emit_unit(&mut self, key: Key) -> slog::Result {
                        self.add(key, "()")
                    }
                }

                let mut ser = Kvencode::default();
                record.kv().serialize(record, &mut ser).unwrap();
                let kvs = ser.finish();

                let mut ser = Kvencode::default();
                values.serialize(record, &mut ser).unwrap();
                let owned = ser.finish();

                let mut string = self.string.lock().unwrap();
                *string += &format!("{}{}{}", record.msg(), kvs, owned);
                Ok(())
            }
        }

        let buffer = Buffer::default();
        let hub = Hub::default().log(slog::Logger::root(buffer.clone().fuse(), slog::o!()));

        struct X;
        impl EventHandler for X {}
        impl Subscriber<Hub> for X {
            type Input = ();
            fn build(_: Hub, _: Self::Input) -> Self {
                Self
            }
            fn subscribe(_: &Hub, _: Shared<Self>) {}
        }

        hub.subscribe::<X>(());
        hub.event.emit(|_| {});

        assert_eq!(
            *buffer.string.lock().unwrap(),
            "Object constructed, listens={}, emissions={}"
        );
    }

    #[test]
    fn removing_an_element() {
        pub trait EventHandler {
            fn remove_me(&self) -> bool;
        }

        hub! {
            Hub {
                event: dyn EventHandler,
            }
        }

        struct X;
        impl EventHandler for X {
            fn remove_me(&self) -> bool {
                true
            }
        }
        impl Subscriber<Hub> for X {
            type Input = ();
            fn build(_: Hub, _: Self::Input) -> Self {
                Self
            }
            fn subscribe(hub: &Hub, shared: Shared<Self>) {
                hub.event.subscribe(shared);
            }
        }

        struct Y;
        impl EventHandler for Y {
            fn remove_me(&self) -> bool {
                false
            }
        }
        impl Subscriber<Hub> for Y {
            type Input = ();
            fn build(_: Hub, _: Self::Input) -> Self {
                Self
            }
            fn subscribe(hub: &Hub, shared: Shared<Self>) {
                hub.event.subscribe(shared);
            }
        }

        let hub = Hub::default();

        hub.subscribe::<X>(());
        hub.subscribe::<X>(());
        hub.subscribe::<Y>(());

        let mut count = 0;
        hub.event.emit(|_| {
            count += 1;
        });
        assert_eq!(count, 3);

        hub.event.filter(|x| x.remove_me());

        let mut count = 0;
        hub.event.emit(|_| {
            count += 1;
        });
        assert_eq!(count, 1);

        hub.event.filter(|x| !x.remove_me());

        let mut count = 0;
        hub.event.emit(|_| {
            count += 1;
        });
        assert_eq!(count, 0);
    }
}
