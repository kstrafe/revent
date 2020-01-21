//! Event broker library for Rust.
//!
//! Implements a synchronous event broker that does not violate mutability constraints.
//! It does so by performing DAG traversals at initialization-time to ensure that no signal
//! chains are able to form a loop.
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
//! use revent::{hub, Subscriber};
//!
//! // Construct a trait for some type of event.
//! pub trait MyEventHandler {
//!     fn my_event(&mut self);
//! }
//!
//! // Create an event hub.
//! // This event hub is the "top level" event hub, it is owned by the calling code and must list
//! // all events that this system will have. It is a comma-separaetd list of the form `name: type`.
//! hub! {
//!     Hub {
//!         event: dyn MyEventHandler,
//!     }
//! }
//!
//! // Create a derivative event hub.
//! // This event hub specifies which of the "top level" event hub signals it wishes to signal
//! // and subscribe to.
//! hub! {
//!     // Name of the new derivative hub.
//!     XHub: Hub {
//!     //    ^^^ - Hub to base itself of, these can also be other derivative event hubs. Is a
//!     // list of comma-separated hubs.
//!         // Here comes a list of signals we want to notify, currently there are none.
//!     } subscribe X {
//!     //          ^ - Structure to bind to this derivative hub.
//!         event,
//!     //  ^^^^^ - We make X subscribe to the `event` channel.
//!     }
//! }
//!
//! // Construct a top-level hub object.
//! let mut hub = Hub::default();
//!
//! // Implement a subscriber to some event.
//! struct X;
//! impl MyEventHandler for X {
//!     fn my_event(&mut self) {
//!         println!("Hello world");
//!     }
//! }
//! impl Subscriber for X {
//!     // Connect X to the XHub created by the macro.
//!     type Hub = XHub;
//!     type Input = ();
//!     fn build(_: Self::Hub, input: Self::Input) -> Self {
//!         Self
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
//! See the `examples` directory for more.
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

/// Generate an event hub or a derivative and its associated boilerplate code.
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
/// let mut my_hub = MyHub::default();
/// // or
/// let mut my_hub = MyHub::new();
///
/// my_hub.channel_name1.emit(|_| {
///     // Do something with each subscriber of `channel_name1`.
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
            // TODO: When gensyms are supported make this symbol a gensym.
            #[doc(hidden)]
            pub _revent_1_manager: $crate::Shared<$crate::Manager>,
            $(
                /// Channel for the given type of event handler.
                pub $channel: $crate::Topic<$type>
            ),*
        }

        impl ::std::default::Default for $hub {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $hub {
            /// Create a new hub.
            pub fn new() -> Self {
                let mng = $crate::Shared::new($crate::Manager::default());
                $(
                    let $channel = $crate::Topic::new(stringify!($channel), &mng);
                )*
                Self {
                    _revent_1_manager: mng,
                    $($channel),*
                }
            }

            /// Set a logger for this hub.
            #[allow(dead_code)]
            #[cfg(feature = "slog")]
            pub fn log(self, logger: slog::Logger) -> Self {
                unsafe { &mut *self.manager().get() }.log = logger;
                self
            }

            /// Insert a subscriber into the hub.
            #[allow(dead_code)]
            pub fn subscribe<T: $crate::Subscriber + $crate::Selfscriber<Self>>(&mut self, input: T::Input)
                where T::Hub: for<'a> ::std::convert::TryFrom<&'a Self, Error = ()>,
            {
                unsafe { &mut *self.manager().get() }.begin_construction();
                let hub: T::Hub = match ::std::convert::TryInto::try_into(&*self) {
                    Ok(hub) => hub,
                    Err(()) => panic!("Internal error: Unable to construct sub-hub."),
                };
                let shared = $crate::Shared::new(T::build(hub, input));
                $crate::Selfscriber::subscribe(self, shared);
                unsafe { &mut *self.manager().get() }.end_construction();
            }

            /// Generate a graphviz (dot) style graph.
            #[allow(dead_code)]
            pub fn graph(&self) -> String {
                unsafe { &mut *self.manager().get() }.graphviz()
            }

            #[doc(hidden)]
            pub unsafe fn manager(&self) -> $crate::Shared<$crate::Manager> {
                self._revent_1_manager.clone()
            }
        }
    };


    ($hub:ident: $derives:ty $(,)? { $($channel:ident: $type:ty),*$(,)? }
     subscribe $structure:ident { $($subscription:ident),*$(,)? }) => {
        hub! {
            $hub { $($channel: $type),* }
        }

        hub! { subscriber $structure, $derives, $($subscription),* }

        impl ::std::convert::TryFrom<&$derives> for $hub {
            type Error = ();
            fn try_from(hub: &$derives) -> ::std::result::Result<Self, Self::Error> {
                Ok(
                    unsafe {
                        Self {
                            _revent_1_manager: hub.manager(),
                            $($channel: hub.$channel.clone_activate()),*
                        }
                    }
                )
            }
        }
    };

    ($hub:ident: $derives:ty, $($rest:ty),+ $(,)? { $($channel:ident: $type:ty),*$(,)? }
     subscribe $structure:ident { $($subscription:ident),*$(,)? }) => {
        hub! {
            $hub: $($rest),* { $($channel: $type),* } subscribe $structure { $($subscription),* }
        }

        hub! { subscriber $structure, $derives, $($subscription),* }

        impl ::std::convert::TryFrom<&$derives> for $hub {
            type Error = ();
            fn try_from(hub: &$derives) -> ::std::result::Result<Self, Self::Error> {
                Ok(
                    unsafe {
                        Self {
                            _revent_1_manager: hub.manager(),
                            $($channel: hub.$channel.clone_activate()),*
                        }
                    }
                )
            }
        }
    };

    (subscriber $structure:ident, $derivative:ty,) => {
        impl $crate::Selfscriber<$derivative> for $structure {
            fn subscribe(_: &mut $derivative, _: $crate::Shared<Self>) {}
        }
    };

    (subscriber $structure:ident, $derivative:ty, $($subscriptions:ident),*) => {
        impl $crate::Selfscriber<$derivative> for $structure {
            fn subscribe(hub: &mut $derivative, shared: $crate::Shared<Self>) {
                unsafe {
                    $(
                        hub.$subscriptions.subscribe(shared.clone());
                    )*
                }
            }
        }
    };
}

/// Implements channel subscription for a given hub.
///
/// Automatically implemented for all hub dependencies by the [hub] macro. It is highly advised to
/// use the macro instead of implementing this trait directly. This trait is public as an artifact
/// of the macro requiring public access.
pub trait Selfscriber<T> {
    /// Subscribe `Self` to various channels of a hub.
    fn subscribe(hub: &mut T, shared: Shared<Self>);
}

/// Subscriber to an event hub.
///
/// Is used by the `subscribe` function generated by the [hub] macro.
pub trait Subscriber
where
    Self: Sized,
{
    /// Hub type to associate with the subscriber.
    type Hub;
    /// Input data to the build function.
    type Input;
    /// Build an instance of the subscriber.
    fn build(hub: Self::Hub, input: Self::Input) -> Self;
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

        hub! {
            XHub: Hub {
            } subscribe X {
                event,
            }
        }

        let mut hub = Hub::default();

        struct X;
        impl EventHandler for X {}
        impl Subscriber for X {
            type Hub = XHub;
            type Input = ();
            fn build(_: Self::Hub, _: Self::Input) -> Self {
                Self
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
    #[should_panic(expected = "Recursion detected: [\"event1\", \"event2\"]")]
    fn transitive_recursion() {
        pub trait EventHandler {}

        hub! {
            Hub {
                event1: dyn EventHandler,
                event2: dyn EventHandler,
            }
        }

        hub! {
            XHub: Hub {
                event1: dyn EventHandler,
            } subscribe X {
                event2,
            }
        }

        hub! {
            YHub: Hub {
                event2: dyn EventHandler,
            } subscribe Y {
                event1,
            }
        }

        let mut hub = Hub::default();

        struct X;
        impl EventHandler for X {}
        impl Subscriber for X {
            type Hub = XHub;
            type Input = ();
            fn build(_: Self::Hub, _: Self::Input) -> Self {
                Self
            }
        }

        struct Y;
        impl EventHandler for Y {}
        impl Subscriber for Y {
            type Hub = YHub;
            type Input = ();
            fn build(_: Self::Hub, _: Self::Input) -> Self {
                Self
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

        let mut hub = Hub::default();

        hub! {
            XHub: Hub {
                event2: dyn EventHandler,
            } subscribe X {
                event1,
            }
        }
        struct X {
            hub: XHub,
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
        impl Subscriber for X {
            type Hub = XHub;
            type Input = Rc<Cell<usize>>;
            fn build(hub: Self::Hub, called: Self::Input) -> Self {
                Self { hub, called }
            }
        }

        hub! {
            YHub: Hub {
            } subscribe Y {
                event1,
                event2,
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
        impl Subscriber for Y {
            type Hub = YHub;
            type Input = Rc<Cell<usize>>;
            fn build(_: Self::Hub, called: Self::Input) -> Self {
                Self { called }
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

        hub! {
            XHub: Hub {
            } subscribe X {
            }
        }

        let mut hub = Hub::default();

        struct X {
            dropped: Rc<Cell<bool>>,
        }
        impl EventHandler for X {}
        impl Subscriber for X {
            type Hub = XHub;
            type Input = Rc<Cell<bool>>;
            fn build(_: Self::Hub, input: Self::Input) -> Self {
                Self { dropped: input }
            }
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
        let mut hub = Hub::default().log(slog::Logger::root(buffer.clone().fuse(), slog::o!()));

        hub! {
            XHub: Hub {
            } subscribe X {
                event,
            }
        }
        struct X;
        impl EventHandler for X {}
        impl Subscriber for X {
            type Hub = XHub;
            type Input = ();
            fn build(_: Self::Hub, _: Self::Input) -> Self {
                Self
            }
        }

        hub.subscribe::<X>(());
        hub.event.emit(|_| {});

        assert_eq!(
            *buffer.string.lock().unwrap(),
            "Object constructed, listens={\"event\"}, emissions={}"
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

        hub! {
            XHub: Hub {
            } subscribe X {
                event,
            }
        }

        hub! {
            YHub: Hub {
            } subscribe Y {
                event,
            }
        }
        struct X;
        impl EventHandler for X {
            fn remove_me(&self) -> bool {
                true
            }
        }
        impl Subscriber for X {
            type Hub = XHub;
            type Input = ();
            fn build(_: Self::Hub, _: Self::Input) -> Self {
                Self
            }
        }

        struct Y;
        impl EventHandler for Y {
            fn remove_me(&self) -> bool {
                false
            }
        }
        impl Subscriber for Y {
            type Hub = YHub;
            type Input = ();
            fn build(_: Self::Hub, _: Self::Input) -> Self {
                Self
            }
        }

        let mut hub = Hub::default();

        hub.subscribe::<X>(());
        hub.subscribe::<X>(());
        hub.subscribe::<Y>(());

        let mut count = 0;
        hub.event.emit(|_| {
            count += 1;
        });
        assert_eq!(count, 3);

        hub.event.remove(|x| x.remove_me());

        let mut count = 0;
        hub.event.emit(|_| {
            count += 1;
        });
        assert_eq!(count, 1);

        hub.event.remove(|x| !x.remove_me());

        let mut count = 0;
        hub.event.emit(|_| {
            count += 1;
        });
        assert_eq!(count, 0);
    }

    #[test]
    fn local_default_trait() {
        trait Default {}
        hub! {
            Hub {
            }
        }
    }

    #[test]
    fn basic_hub_is_send() {
        pub trait A: Send {}
        hub! {
            Hub {
                signal: dyn A,
            }
        }
        let mut hub = Hub::new();
        std::thread::spawn(move || {
            hub.signal.emit(|_| {});
        });
    }
}
