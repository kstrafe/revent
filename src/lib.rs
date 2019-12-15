//! Mutable aliasing free event system for Rust.
//!
//! # What is an event system? #
//!
//! An event system is a system where an object can create an event, and any other object
//! (including the one generating the event itself) can listen and react to this event and update its state.
//!
//! # Why do we want this? #
//!
//! It allows us to decouple objects that need to uphold some invariant with respect to each
//! other.
//!
//! Take for instance a video player. If it loads a video then it should probably update the
//! run-time of that video in the GUI. So the video loader can emit an event of the type
//! `VideoLoaded` which the GUI listens to and updates its state.
//!
//! The alternative is to somehow encode this ad-hoc, by calling an update function for the GUI
//! inside the video loader. This becomes unwieldy in large programs.
//!
//! # Example of basic usage #
//!
//! ```
//! use revent::{down, Event, EventStore, Notifiable};
//! use std::any::TypeId;
//!
//! struct MyNotifiable;
//!
//! impl Notifiable for MyNotifiable {
//!     fn notify(&mut self, event: &dyn Event, _: &mut EventStore) {
//!         println!("I am notified");
//!         if event.type_id() == TypeId::of::<u32>() {
//!             println!("This is a u32");
//!             // Do something useful...
//!         }
//!
//!         // Downcasting using the utility down function
//!         if let Some(value) = down::<u32>(event) {
//!             println!("Access to the u32 value: {}", value);
//!         }
//!     }
//! }
//!
//! let mut mn = MyNotifiable { };
//!
//! mn.with_notify(|this, store| {
//!     store.emit(123u32);
//! });
//! ```
//!
//! The order in which events are processed is FIFO (first-in, first-out). Meaning that emitting an
//! event guarantees that events emitted before will be run before.
//!
//! # More information #
//!
//! This library imagines a program or library using `revent` to be a nested structure of structs,
//! many of which implement [Notifiable]. If a struct wishes to notify itself and its
//! sub-structures, it should use `self.notify`. If it wishes to notify parents to the Nth degree
//! it should [EventStore::emit] into an `EventStore`.
//!
//! ```
//! // An example of direct self-notification
//! use revent::{Event, EventStore, Notifiable};
//! use std::any::TypeId;
//!
//! struct MyNotifiable;
//!
//! impl Notifiable for MyNotifiable {
//!     fn notify(&mut self, event: &dyn Event, _: &mut EventStore) {
//!         println!("Notified");
//!     }
//! }
//!
//! impl MyNotifiable {
//!     pub fn do_something(&mut self) {
//!         self.with_notify(|this, store| {
//!             // We need a store here because `notify` takes one, to which it itself can add
//!             // events.
//!             this.notify(&0u32, store);
//!         });
//!     }
//! }
//!
//! let mut mn = MyNotifiable { };
//!
//! mn.do_something();
//! ```
//!
//! The following shows how substructures can emit events to super-structures without themselves
//! being [Notifiable].
//!
//! ```
//! // An example of emitting notifications to some super-structure.
//! use revent::{Event, EventStore, Notifiable};
//! use std::any::TypeId;
//!
//! struct MyNotifiable {
//!     substructure: Substructure,
//! }
//!
//! impl Notifiable for MyNotifiable {
//!     fn notify(&mut self, event: &dyn Event, store: &mut EventStore) {
//!         println!("Notified");
//!     }
//! }
//!
//! impl MyNotifiable {
//!     pub fn do_something(&mut self) {
//!         self.with_notify(|this, store| {
//!             this.substructure.do_substructure_thing(store);
//!         });
//!     }
//! }
//!
//!
//! struct Substructure;
//!
//! impl Substructure {
//!     pub fn do_substructure_thing(&mut self, store: &mut EventStore) {
//!         store.emit(0u32);
//!     }
//! }
//!
//! let mut mn = MyNotifiable {
//!     substructure: Substructure { },
//! };
//!
//! mn.do_something();
//! ```
//!
//! # Downsides #
//!
//! Emitting events into the `EventStore` does not immediately execute an event. This is
//! unfortunately the way it is due to Rusts aliasing rules: We simply cannot hold a
//! super-structure while also mutably borring a field of that struct. We thus must accumulate
//! events into a buffer (`EventStore`) and execute this store when control returns to the
//! super-structure.
#![deny(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]
use std::{any::Any, collections::VecDeque};

/// A generic event. Implemented for all types.
pub trait Event: Any {
    /// Get the reference to this events [Any].
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> Event for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Shorthand version for downcasting an [Event].
pub fn down<T: 'static>(event: &dyn Event) -> Option<&T> {
    Any::downcast_ref::<T>(event.as_any())
}

/// Event storage. Events are [EventStore::emit]ted into this structure.
#[derive(Default)]
pub struct EventStore {
    pub(crate) store: VecDeque<Box<dyn Event>>,
}

impl EventStore {
    /// Add an event to the event storage.
    pub fn emit<T: 'static + Event>(&mut self, event: T) {
        self.store.push_back(Box::new(event));
    }
}

/// Main trait of this crate to implement on structures.
pub trait Notifiable {
    /// Notify this structure of an event. It should call [Notifiable::notify] on all subclasses that are
    /// [Notifiable].
    fn notify(&mut self, event: &dyn Event, store: &mut EventStore);

    /// Takes event storage from this struct. Used for optimizing away the creation and destruction
    /// of the event store. By doing so, we can reuse the same event store for all events.
    ///
    /// The default implementation creates a new event store. If you want to speed up your program
    /// then you need to take the event store from the struct (via [Option::take]). Note that you
    /// must then also use [Notifiable::set_storage] to put the store back after it has been used.
    fn take_storage(&mut self) -> EventStore {
        EventStore::default()
    }

    /// Return the event store to its place in this struct. See [Notifiable::take_storage] for more
    /// information.
    fn set_storage(&mut self, _: EventStore) {}

    /// Runs code with a given event store and executes all the events at the end of the
    /// scope.
    fn with_notify(&mut self, mut mutator: impl FnMut(&mut Self, &mut EventStore)) {
        let mut events = self.take_storage();
        mutator(self, &mut events);

        while !events.store.is_empty() {
            let event = events.store.pop_front().unwrap();
            self.notify(&*event, &mut events);
        }

        self.set_storage(events);
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::any::TypeId;

    struct EmptyEvent;

    #[test]
    fn self_notification() {
        struct Example {
            seen_event: bool,
        }

        impl Notifiable for Example {
            fn notify(&mut self, event: &dyn Event, _: &mut EventStore) {
                if event.type_id() == TypeId::of::<EmptyEvent>() {
                    self.seen_event = true;
                }
            }
        }

        // ---

        let mut example = Example { seen_event: false };

        assert!(!example.seen_event);

        example.with_notify(|_, store| {
            store.emit(EmptyEvent {});
        });

        assert!(example.seen_event);
    }

    #[test]
    fn substructure_access() {
        struct Substructure {
            seen_event: bool,
        }

        impl Notifiable for Substructure {
            fn notify(&mut self, event: &dyn Event, _: &mut EventStore) {
                if event.type_id() == TypeId::of::<EmptyEvent>() {
                    self.seen_event = true;
                }
            }
        }

        impl Substructure {
            fn generate_event(&mut self, store: &mut EventStore) {
                store.emit(EmptyEvent {});
            }
        }

        // ---

        struct Example {
            seen_event: bool,
            substructure: Substructure,
        }

        impl Notifiable for Example {
            fn notify(&mut self, event: &dyn Event, store: &mut EventStore) {
                if event.type_id() == TypeId::of::<EmptyEvent>() {
                    self.seen_event = true;
                }
                self.substructure.notify(event, store);
            }
        }

        // ---

        let mut example = Example {
            seen_event: false,
            substructure: Substructure { seen_event: false },
        };

        assert!(!example.seen_event);
        assert!(!example.substructure.seen_event);

        example.with_notify(|example, store| {
            example.substructure.generate_event(store);
        });

        assert!(example.seen_event);
        assert!(example.substructure.seen_event);
    }

    #[test]
    fn recursive_events() {
        struct ReactiveEvent;

        struct Example {
            seen_reactive_event: bool,
        }

        impl Notifiable for Example {
            fn notify(&mut self, event: &dyn Event, store: &mut EventStore) {
                if event.type_id() == TypeId::of::<EmptyEvent>() {
                    assert!(!self.seen_reactive_event);
                    store.emit(ReactiveEvent {});
                } else if event.type_id() == TypeId::of::<ReactiveEvent>() {
                    self.seen_reactive_event = true;
                }
            }
        }

        // ---

        let mut example = Example {
            seen_reactive_event: false,
        };

        assert!(!example.seen_reactive_event);

        example.with_notify(|_, store| {
            store.emit(EmptyEvent {});
        });

        assert!(example.seen_reactive_event);
    }

    #[test]
    fn multiple_events() {
        struct Example {
            seen_events: u8,
        }

        impl Notifiable for Example {
            fn notify(&mut self, event: &dyn Event, _: &mut EventStore) {
                if event.type_id() == TypeId::of::<EmptyEvent>() {
                    self.seen_events += 1;
                }
            }
        }

        // ---

        let mut example = Example { seen_events: 0 };

        assert_eq!(0, example.seen_events);

        example.with_notify(|this, store| {
            store.emit(EmptyEvent {});
            assert_eq!(0, this.seen_events);
            store.emit(EmptyEvent {});
            assert_eq!(0, this.seen_events);
            store.emit(EmptyEvent {});
            assert_eq!(0, this.seen_events);
        });

        assert_eq!(3, example.seen_events);
    }

    #[test]
    fn downcasting_event() {
        struct NumberEvent {
            value: u8,
        }

        struct Example {
            number: u8,
        }

        impl Notifiable for Example {
            fn notify(&mut self, event: &dyn Event, _: &mut EventStore) {
                if let Some(NumberEvent { value }) = down(event) {
                    self.number = *value;
                }
            }
        }

        // ---

        let mut example = Example { number: 0 };

        assert_eq!(0, example.number);

        example.with_notify(|_, store| {
            store.emit(EmptyEvent {});
            store.emit(NumberEvent { value: 13 });
            store.emit(EmptyEvent {});
            store.emit(NumberEvent { value: 123 });
        });

        assert_eq!(123, example.number);
    }
}
