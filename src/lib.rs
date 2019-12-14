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
//! use revent::{Event, EventStore, Notifiable};
//! use std::any::TypeId;
//!
//! struct MyEvent;
//!
//! impl Event for MyEvent { }
//!
//! struct MyNotifiable;
//!
//! impl Notifiable for MyNotifiable {
//!     fn notify(&mut self, event: &dyn Event, _: &mut EventStore) {
//!         println!("I was notified!");
//!         if event.type_id() == TypeId::of::<MyEvent>() {
//!             println!("This was MyEvent!");
//!             // Do something useful...
//!         }
//!     }
//! }
//!
//! let mut mn = MyNotifiable { };
//!
//! mn.with_notify(|this, store| {
//!     store.emit(MyEvent { });
//! });
//! ```
//!
//! The order in which events are processed is FIFO (first-in, first-out). Meaning that emitting an
//! event guarantees that events emitted before will be run before.
#![deny(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]
use std::{any::Any, collections::VecDeque};

/// A generic event.
pub trait Event: Any {}

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
        self.notify_all(&mut events);
        self.set_storage(events);
    }

    /// Notifies this structure for each element stored in the [EventStore].
    fn notify_all(&mut self, events: &mut EventStore) {
        while !events.store.is_empty() {
            let event = events.store.pop_front().unwrap();
            self.notify(&*event, events);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::any::TypeId;

    struct EmptyEvent;

    impl Event for EmptyEvent {}

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

        impl Event for ReactiveEvent {}

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
}
