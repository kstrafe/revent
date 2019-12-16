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
//! use revent::{Event, Notifiable};
//!
//! struct X;
//!
//! impl Notifiable for X {
//!     fn event(&mut self, _event: &dyn Event, _system: &mut dyn Notifiable) {
//!         println!("An event has been received!");
//!         // From here, call `event` on all direct children in X
//!     }
//! }
//!
//! let mut x = X;
//!
//! let system = &mut ();
//!
//! x.notify(&"This is an event, almost anything can be an event".to_string(), system);
//! ```
//!
//! # Nested structures #
//!
//! When dealing with nested structures, we want to our notifications to be sent to every
//! [Notifiable] object. Because of Rust's mutable aliasing restriction this is not as
//! straighforward as just putting the object in a list.
//!
//! Instead what we do is we move a notifier out of the structure tree and consider the rest of the
//! tree as one single [Notifier]. This way, the structure that was split out can call
//! `self.notify` with the rest of the tree as the system - causing all notifiables to be updated.
//!
//! ```
//! use revent::{down, Event, Notifiable, Notifier};
//!
//! // We make 3 classes as exemplified by the video player introduction above
//!
//! struct Client {
//!     gui: Notifier<Gui>,
//!     video: Notifier<Video>,
//! }
//! // Contains data we use to draw visual elements to the screen
//! struct Gui {
//!     pub running_time: u32,
//! }
//! struct Video; // Contains the video loader, decoder, and so on
//!
//! // Make all these notifiable
//!
//! // Note that `system` means "the rest of the structures", so it excludes self.
//!
//! impl Notifiable for Client {
//!     fn event(&mut self, event: &dyn Event, system: &mut dyn Notifiable) {
//!         println!("Client got an event");
//!         self.gui.event(event, system);
//!         self.video.event(event, system);
//!     }
//! }
//!
//! impl Notifiable for Gui {
//!     fn event(&mut self, event: &dyn Event, system: &mut dyn Notifiable) {
//!         println!("Gui got an event");
//!         if let Some(VideoChanged { new_time }) = down(event) {
//!             self.running_time = *new_time;
//!         }
//!     }
//! }
//!
//! impl Notifiable for Video {
//!     fn event(&mut self, event: &dyn Event, system: &mut dyn Notifiable) {
//!         println!("Video got an event");
//!     }
//! }
//!
//! // Add some functions that do work
//!
//! impl Client {
//!     fn client_work(&mut self, system: &mut dyn Notifiable) {
//!         // Let's load a new video, as per the introductory paragraph
//!         Notifier::split(
//!             self,
//!             |x| &mut x.video,
//!             |system, video| {
//!                 video.video_work(system);
//!             },
//!         );
//!     }
//! }
//!
//! impl Gui {
//!     fn gui_work(&mut self, system: &mut dyn Notifiable) {
//!     }
//! }
//!
//! impl Video {
//!     fn video_work(&mut self, system: &mut dyn Notifiable) {
//!         // Do some loading work...
//!         let new_time = 123;
//!         self.notify(&VideoChanged { new_time }, system);
//!     }
//! }
//!
//! // A message that Video can send
//! #[derive(Clone)]
//! struct VideoChanged {
//!     pub new_time: u32,
//! }
//!
//! // Create a client
//! let mut client = Client { gui: Notifier::new(Gui { running_time: 0 }), video: Notifier::new(Video) };
//!
//! // By making the root system `&mut ()` we're essentially saying that the events stop here, we
//! // have nowhere to send them to in this context (in `fn main`).
//! let mut root_system = &mut ();
//!
//! // Let's make sure the Gui's running time starts at 0
//! assert_eq!(client.gui.running_time, 0);
//!
//! // To simulate the introductory paragraph, let's load a new video
//! client.client_work(root_system);
//!
//! // Because we loaded a new video, the Gui's event handler should have updated its own state
//! // accordingly
//!
//! assert_eq!(client.gui.running_time, 123);
//! ```
#![deny(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]
use std::{
    any::Any,
    collections::VecDeque,
    ops::{Deref, DerefMut},
};

/// A generic event. Implemented for all types.
pub trait Event: Any + 'static {
    /// Get the reference to this events [Any].
    fn as_any(&self) -> &dyn Any;
    /// da
    fn as_box(&self) -> Box<dyn Event + 'static>;
}

impl<T: Any + Clone> Event for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_box(&self) -> Box<dyn Event + 'static> {
        Box::new(self.clone())
    }
}

struct EventStore {
    store: VecDeque<Box<dyn Event>>,
}

impl EventStore {
    fn new() -> Self {
        Self {
            store: VecDeque::new(),
        }
    }

    fn pop(&mut self) -> Option<Box<dyn Event>> {
        self.store.pop_front()
    }
}

impl Notifiable for EventStore {
    fn event(&mut self, event: &dyn Event, _: &mut dyn Notifiable) {
        self.store.push_back(event.as_box());
    }

    fn notify(&mut self, _: &dyn Event, _: &mut dyn Notifiable) {
        panic!("The event store cannot be notified");
    }
}

/// Shorthand version for downcasting an [Event].
pub fn down<T: 'static>(event: &dyn Event) -> Option<&T> {
    Any::downcast_ref::<T>(event.as_any())
}

/// Main trait of this crate to implement on structures.
pub trait Notifiable {
    /// Handle the event for this structure. Call [Notifiable::notify] instead of this.
    ///
    /// What you should do: In this method delegate the event down to all fields that are
    /// [Notifiable] and perform any internal changes to the structure to reflect the event.
    fn event(&mut self, event: &dyn Event, system: &mut dyn Notifiable);

    /// Notify this structure and the system about an event.
    ///
    /// Calls [Notifiable::event] on both the current object and the system.
    fn notify(&mut self, event: &dyn Event, system: &mut dyn Notifiable) {
        let mut store = EventStore::new();
        system.event(event, &mut store);
        self.event(event, &mut store);

        while let Some(event) = store.pop() {
            system.event(&*event, &mut store);
            self.event(&*event, &mut store);
        }
    }
}

impl Notifiable for () {
    fn event(&mut self, _: &dyn Event, _: &mut dyn Notifiable) {}
}

/// Wrapper structure for notifiers.
///
/// When a structure sends a notification, it must be split out from the tree structure it
/// originates from. The reason for this is twofold:
///
/// 1. To avoid double-self notification.
pub struct Notifier<T: Notifiable>(Option<T>);

impl<T: Notifiable> Notifier<T> {
    /// Create a new [Notifier].
    pub fn new(datum: T) -> Self {
        Self(Some(datum))
    }

    /// Access a notifier inside a structure and supply the parent structure as the system.
    pub fn split<O: Notifiable>(
        datum: &mut O,
        mut accessor: impl FnMut(&mut O) -> &mut Notifier<T>,
        mut mutator: impl FnMut(&mut dyn Notifiable, &mut T),
    ) {
        let access = accessor(datum);
        let mut notifier = access.0.take().unwrap();

        mutator(datum, &mut notifier);

        let access = accessor(datum);
        access.0 = Some(notifier);
    }
}

impl<T: Notifiable> Deref for Notifier<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<T: Notifiable> DerefMut for Notifier<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}

impl<T: Notifiable> Notifiable for Notifier<T> {
    fn event(&mut self, event: &dyn Event, system: &mut dyn Notifiable) {
        if let Some(ref mut item) = self.0 {
            item.event(event, system);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::any::TypeId;

    #[derive(Clone)]
    struct EmptyEvent;

    #[test]
    fn self_notification() {
        struct Example {
            seen_event: bool,
        }

        impl Notifiable for Example {
            fn event(&mut self, event: &dyn Event, _: &mut dyn Notifiable) {
                if event.type_id() == TypeId::of::<EmptyEvent>() {
                    self.seen_event = true;
                }
            }
        }

        // ---

        let mut example = Example { seen_event: false };

        assert!(!example.seen_event);

        example.notify(&EmptyEvent, &mut ());

        assert!(example.seen_event);
    }

    #[test]
    fn substructure_access() {
        struct Substructure {
            seen_event: bool,
        }

        impl Notifiable for Substructure {
            fn event(&mut self, event: &dyn Event, _: &mut dyn Notifiable) {
                if event.type_id() == TypeId::of::<EmptyEvent>() {
                    self.seen_event = true;
                }
            }
        }

        impl Substructure {
            fn generate_event(&mut self, system: &mut dyn Notifiable) {
                self.notify(&EmptyEvent {}, system);
            }
        }

        // ---

        struct Example {
            seen_event: bool,
            substructure: Notifier<Substructure>,
        }

        impl Notifiable for Example {
            fn event(&mut self, event: &dyn Event, system: &mut dyn Notifiable) {
                if event.type_id() == TypeId::of::<EmptyEvent>() {
                    self.seen_event = true;
                }
                self.substructure.event(event, system);
            }
        }

        // ---

        let mut example = Example {
            seen_event: false,
            substructure: Notifier::new(Substructure { seen_event: false }),
        };

        assert!(!example.seen_event);
        assert!(!example.substructure.seen_event);

        Notifier::split(
            &mut example,
            |x| &mut x.substructure,
            |system, substructure| {
                substructure.generate_event(system);
            },
        );

        assert!(example.seen_event);
        assert!(example.substructure.seen_event);
    }

    #[test]
    fn recursive_events() {
        #[derive(Clone)]
        struct ReactiveEvent;

        struct Example {
            seen_reactive_event: bool,
        }

        impl Notifiable for Example {
            fn event(&mut self, event: &dyn Event, system: &mut dyn Notifiable) {
                if event.type_id() == TypeId::of::<EmptyEvent>() {
                    assert!(!self.seen_reactive_event);
                    self.notify(&ReactiveEvent {}, system);
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

        example.notify(&EmptyEvent {}, &mut ());

        assert!(example.seen_reactive_event);
    }

    #[test]
    fn multiple_events() {
        struct Example {
            seen_events: u8,
        }

        impl Notifiable for Example {
            fn event(&mut self, event: &dyn Event, _: &mut dyn Notifiable) {
                if event.type_id() == TypeId::of::<EmptyEvent>() {
                    self.seen_events += 1;
                }
            }
        }

        // ---

        let mut example = Example { seen_events: 0 };

        assert_eq!(0, example.seen_events);

        example.notify(&EmptyEvent {}, &mut ());
        assert_eq!(1, example.seen_events);

        example.notify(&EmptyEvent {}, &mut ());
        assert_eq!(2, example.seen_events);

        example.notify(&EmptyEvent {}, &mut ());
        assert_eq!(3, example.seen_events);
    }

    #[test]
    fn downcasting_event() {
        #[derive(Clone)]
        struct NumberEvent {
            value: u8,
        }

        struct Example {
            number: u8,
        }

        impl Notifiable for Example {
            fn event(&mut self, event: &dyn Event, _: &mut dyn Notifiable) {
                if let Some(NumberEvent { value }) = down(event) {
                    self.number = *value;
                }
            }
        }

        // ---

        let mut example = Example { number: 0 };

        assert_eq!(0, example.number);

        example.notify(&EmptyEvent {}, &mut ());
        example.notify(&NumberEvent { value: 13 }, &mut ());
        example.notify(&EmptyEvent {}, &mut ());
        example.notify(&NumberEvent { value: 123 }, &mut ());

        assert_eq!(123, example.number);
    }
}
