//! Synchronous event system.
//!
//! # What is an event system? #
//!
//! An event system is a set of slots which contain objects. A signal is emitted on a slot, which
//! will call each object in the slot. Invoked objects can then send more signals to different
//! slots.
//!
//! # Synchronous #
//!
//! Revent's events are synchronous, meaning that emitting an event will immediately process all
//! handlers in a slot. Once the function call returns, it is guaranteed that all listeners have
//! been called.
//!
//! # Example #
//!
//! ```
//! use revent::{Anchor, Manager, Named, Slot, Subscriber};
//! use std::{cell::RefCell, rc::Rc};
//!
//! trait BasicSignal {}
//!
//! struct Hub {
//!     basic_slot: Slot<dyn BasicSignal>,
//!     mng: Rc<RefCell<Manager>>,
//! }
//! impl Hub {
//!     fn new() -> Self {
//!         let mng = Rc::new(RefCell::new(Manager::default()));
//!         Self {
//!             basic_slot: Slot::new("basic_slot", mng.clone()),
//!             mng,
//!         }
//!     }
//! }
//! impl Anchor for Hub {
//!     fn manager(&self) -> &Rc<RefCell<Manager>> {
//!         &self.mng
//!     }
//! }
//!
//! // ---
//!
//! struct MySubscriber;
//! impl Subscriber<Hub> for MySubscriber {
//!     type Emitter = ();
//!
//!     fn register(hub: &mut Hub, item: Rc<RefCell<Self>>) {
//!         hub.basic_slot.register(item);
//!     }
//! }
//! impl Named for MySubscriber {
//!     const NAME: &'static str = "MySubscriber";
//! }
//! impl BasicSignal for MySubscriber {}
//!
//! // ---
//!
//! let mut hub = Hub::new();
//! let item = hub.subscribe(|_| MySubscriber);
//! hub.basic_slot.emit(|x| {
//!     println!("Called for each subscriber");
//! });
//! hub.unsubscribe(&item);
//! ```
//!
//! ## Mutable cycles ##
//!
//! Revent performs cycle detection in [subscribe](crate::Anchor::subscribe) and ensures that no
//! system exists in which we can create double mutable borrows.
//!
//! # Core Concepts #
//!
//! An event system based on revent has 3 core concepts:
//!
//! * Anchors
//! * Emitters
//! * Subscribers
//!
//! ## Anchor ##
//!
//! An anchor contains all event [Slot]s and [Single]s in the system. It also contains a [Manager] and
//! implements [Anchor]. We register new [Subscriber]s to an anchor, and the subscribers will themselves
//! choose which slots/singles to listen or emit to. Only anchors can be subscribed to.
//!
//! ## Subscriber ##
//!
//! Subscribers are classes that implement `Subscriber<A: Anchor>`. They specify their interest in
//! signals to listen to by `fn register`. They specify their singles/slots to emit to via `type
//! Emitter`.
//!
//! ## Emitter ##
//!
//! Each subscriber has an associated [Emitter](crate::Subscriber::Emitter). An emitter contains a list of singles and slots
//! based on the `Anchor` of its subscriber. Emitters simply implement [Emit] for `Anchor` which
//! clones singles and slots from a particular anchor to the emitter itself.
//!
//! # Example with Emitter #
//!
//! ```
//! use revent::{Anchor, Emit, Manager, Named, Slot, Subscriber};
//! use std::{cell::RefCell, rc::Rc};
//!
//! // First let's crate a hub (Anchor) that contains two signals.
//!
//! trait BasicSignal {
//!     fn basic(&mut self);
//! }
//!
//! struct Hub {
//!     basic_slot_1: Slot<dyn BasicSignal>,
//!     basic_slot_2: Slot<dyn BasicSignal>,
//!     mng: Rc<RefCell<Manager>>,
//! }
//! impl Hub {
//!     fn new() -> Self {
//!         let mng = Rc::new(RefCell::new(Manager::default()));
//!         Self {
//!             basic_slot_1: Slot::new("basic_slot_1", mng.clone()),
//!             basic_slot_2: Slot::new("basic_slot_2", mng.clone()),
//!             mng,
//!         }
//!     }
//! }
//! impl Anchor for Hub {
//!     fn manager(&self) -> &Rc<RefCell<Manager>> {
//!         &self.mng
//!     }
//! }
//!
//! // ---
//!
//! // Now we define our emitter structure, this one contains only `basic_slot_2`, which indicates
//! // that we want to emit only to this slot for the subscribers using it as their emitter.
//!
//! struct MyEmitter {
//!     basic_slot_2: Slot<dyn BasicSignal>,
//! }
//!
//! impl Emit<Hub> for MyEmitter {
//!     fn create(item: &Hub) -> Self {
//!         Self {
//!             basic_slot_2: item.basic_slot_2.clone(),
//!         }
//!     }
//! }
//!
//! // ---
//!
//! // Create a subscriber that uses MyEmitter (emits on `basic_slot_2`), and listens on
//! // `basic_slot_1`.
//!
//! struct MySubscriber { emitter: MyEmitter }
//! impl Subscriber<Hub> for MySubscriber {
//!     // Indicate which emitter we want to use.
//!     type Emitter = MyEmitter;
//!
//!     fn register(hub: &mut Hub, item: Rc<RefCell<Self>>) {
//!         hub.basic_slot_1.register(item);
//!     }
//! }
//! impl Named for MySubscriber {
//!     const NAME: &'static str = "MySubscriber";
//! }
//!
//! // Whenever we get a basic signal we pass it to the emitter.
//! impl BasicSignal for MySubscriber {
//!     fn basic(&mut self) {
//!         self.emitter.basic_slot_2.emit(|_| println!("Hello world"));
//!     }
//! }
//!
//! // ---
//!
//! let mut hub = Hub::new();
//! let item = hub.subscribe(|emitter| MySubscriber { emitter });
//! hub.basic_slot_1.emit(BasicSignal::basic);
//! hub.unsubscribe(&item);
//! ```
//!
#![deny(
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]

mod mng;
mod single;
mod slot;
mod traits;
pub(crate) use self::mng::Mode;
pub use self::{
    mng::{Grapher, Manager},
    single::Single,
    slot::Slot,
    traits::{Anchor, Emit, Named, Subscriber},
};

use std::{cell::RefCell, rc::Rc};

thread_local! {
    static STACK: RefCell<Vec<(Mode, Rc<RefCell<Manager>>)>> = RefCell::new(Vec::new());
}

fn assert_active_manager(manager: &Rc<RefCell<Manager>>) {
    STACK.with(|x| {
        assert!(
            Rc::ptr_eq(
                &x.borrow()
                    .last()
                    .expect("revent: signal modification outside of Anchor context")
                    .1,
                manager
            ),
            "revent: manager is different"
        );
    });
}

#[cfg(test)]
mod tests {
    use crate::{Anchor, Emit, Manager, Named, Single, Slot, Subscriber};
    use std::{cell::RefCell, rc::Rc};

    #[quickcheck_macros::quickcheck]
    fn basic(value: usize) {
        trait BasicSignal {}

        struct Hub {
            basic_signal: Slot<dyn BasicSignal>,
            mng: Rc<RefCell<Manager>>,
        }
        impl Hub {
            fn new() -> Self {
                let mng = Rc::new(RefCell::new(Manager::default()));
                Self {
                    basic_signal: Slot::new("basic_signal", mng.clone()),

                    mng,
                }
            }
        }
        impl Anchor for Hub {
            fn manager(&self) -> &Rc<RefCell<Manager>> {
                &self.mng
            }
        }

        // ---

        struct MySubscriberNode;
        impl Emit<Hub> for MySubscriberNode {
            fn create(_: &Hub) -> Self {
                Self
            }
        }
        struct MySubscriber;
        impl Subscriber<Hub> for MySubscriber {
            type Emitter = MySubscriberNode;

            fn register(hub: &mut Hub, item: Rc<RefCell<Self>>) {
                hub.basic_signal.register(item);
            }
        }
        impl Named for MySubscriber {
            const NAME: &'static str = "MySubscriber";
        }
        impl BasicSignal for MySubscriber {}

        // ---

        let mut hub = Hub::new();

        for _ in 0..value {
            hub.subscribe(|_| MySubscriber);
        }

        let mut count = 0;

        hub.basic_signal.emit(|_| {
            count += 1;
        });

        assert_eq!(value, count);
    }

    #[test]
    #[should_panic(
        expected = "revent: found a recursion during subscription: [MySubscriber]basic_signal -> basic_signal"
    )]
    fn self_subscribing() {
        trait BasicSignal {}

        struct Hub {
            basic_signal: Slot<dyn BasicSignal>,
            mng: Rc<RefCell<Manager>>,
        }
        impl Hub {
            fn new() -> Self {
                let mng = Rc::new(RefCell::new(Manager::default()));
                Self {
                    basic_signal: Slot::new("basic_signal", mng.clone()),
                    mng,
                }
            }
        }
        impl Anchor for Hub {
            fn manager(&self) -> &Rc<RefCell<Manager>> {
                &self.mng
            }
        }

        // ---

        struct MySubscriberNode;
        impl Emit<Hub> for MySubscriberNode {
            fn create(hub: &Hub) -> Self {
                let _ = hub.basic_signal.clone();
                Self
            }
        }
        struct MySubscriber;
        impl Subscriber<Hub> for MySubscriber {
            type Emitter = MySubscriberNode;
            fn register(hub: &mut Hub, item: Rc<RefCell<Self>>) {
                hub.basic_signal.register(item);
            }
        }
        impl Named for MySubscriber {
            const NAME: &'static str = "MySubscriber";
        }
        impl BasicSignal for MySubscriber {}

        // ---

        let mut hub = Hub::new();

        hub.subscribe(|_| MySubscriber);
    }

    #[test]
    #[should_panic(
        expected = "revent: found a recursion during subscription: [MySubscriber]basic_signal -> [OtherSubscriber]other_signal -> basic_signal"
    )]
    fn transitive_self_subscription() {
        trait BasicSignal {}
        trait OtherSignal {}

        struct Hub {
            basic_signal: Slot<dyn BasicSignal>,
            other_signal: Slot<dyn OtherSignal>,
            mng: Rc<RefCell<Manager>>,
        }
        impl Hub {
            fn new() -> Self {
                let mng = Rc::new(RefCell::new(Manager::default()));
                Self {
                    basic_signal: Slot::new("basic_signal", mng.clone()),
                    other_signal: Slot::new("other_signal", mng.clone()),
                    mng,
                }
            }
        }
        impl Anchor for Hub {
            fn manager(&self) -> &Rc<RefCell<Manager>> {
                &self.mng
            }
        }

        // ---

        struct MySubscriberNode;
        impl Emit<Hub> for MySubscriberNode {
            fn create(hub: &Hub) -> Self {
                let _ = hub.other_signal.clone();
                Self
            }
        }
        struct MySubscriber;
        impl Subscriber<Hub> for MySubscriber {
            type Emitter = MySubscriberNode;
            fn register(hub: &mut Hub, item: Rc<RefCell<Self>>) {
                hub.basic_signal.register(item);
            }
        }
        impl Named for MySubscriber {
            const NAME: &'static str = "MySubscriber";
        }
        impl BasicSignal for MySubscriber {}

        // ---

        struct OtherSubscriberNode;
        impl Emit<Hub> for OtherSubscriberNode {
            fn create(hub: &Hub) -> Self {
                let _ = hub.basic_signal.clone();
                Self
            }
        }
        struct OtherSubscriber;
        impl Subscriber<Hub> for OtherSubscriber {
            type Emitter = OtherSubscriberNode;
            fn register(hub: &mut Hub, item: Rc<RefCell<Self>>) {
                hub.other_signal.register(item);
            }
        }
        impl Named for OtherSubscriber {
            const NAME: &'static str = "OtherSubscriber";
        }
        impl OtherSignal for OtherSubscriber {}

        // ---

        let mut hub = Hub::new();

        hub.subscribe(|_| MySubscriber);
        hub.subscribe(|_| OtherSubscriber);
    }

    #[quickcheck_macros::quickcheck]
    fn register_and_unsubscribe(subscribes: usize) {
        trait BasicSignal {}

        struct Hub {
            basic_signal: Slot<dyn BasicSignal>,
            mng: Rc<RefCell<Manager>>,
        }
        impl Hub {
            fn new() -> Self {
                let mng = Rc::new(RefCell::new(Manager::default()));
                Self {
                    basic_signal: Slot::new("basic_signal", mng.clone()),
                    mng,
                }
            }
        }
        impl Anchor for Hub {
            fn manager(&self) -> &Rc<RefCell<Manager>> {
                &self.mng
            }
        }

        // ---

        struct MySubscriberNode;
        impl Emit<Hub> for MySubscriberNode {
            fn create(_: &Hub) -> Self {
                Self
            }
        }
        struct MySubscriber;
        impl Subscriber<Hub> for MySubscriber {
            type Emitter = MySubscriberNode;
            fn register(hub: &mut Hub, item: Rc<RefCell<Self>>) {
                hub.basic_signal.register(item);
            }
        }
        impl Named for MySubscriber {
            const NAME: &'static str = "MySubscriber";
        }
        impl BasicSignal for MySubscriber {}

        // ---

        let mut hub = Hub::new();

        let mut items = Vec::with_capacity(subscribes);
        for _ in 0..subscribes {
            items.push(hub.subscribe(|_| MySubscriber));
        }

        {
            let mut count = 0;
            hub.basic_signal.emit(|_| {
                count += 1;
            });
            assert_eq!(subscribes, count);
        }

        for item in items.drain(..) {
            hub.unsubscribe::<MySubscriber>(&item);
        }

        {
            let mut count = 0;
            hub.basic_signal.emit(|_| {
                count += 1;
            });
            assert_eq!(0, count);
        }
    }

    #[test]
    #[should_panic(expected = "revent: unable to deregister nonexistent item")]
    fn double_unsubscribe() {
        trait BasicSignal {}

        struct Hub {
            basic_signal: Slot<dyn BasicSignal>,
            mng: Rc<RefCell<Manager>>,
        }
        impl Hub {
            fn new() -> Self {
                let mng = Rc::new(RefCell::new(Manager::default()));
                Self {
                    basic_signal: Slot::new("basic_signal", mng.clone()),
                    mng,
                }
            }
        }
        impl Anchor for Hub {
            fn manager(&self) -> &Rc<RefCell<Manager>> {
                &self.mng
            }
        }

        // ---

        struct MySubscriberNode;
        impl Emit<Hub> for MySubscriberNode {
            fn create(_: &Hub) -> Self {
                Self
            }
        }
        struct MySubscriber;
        impl Subscriber<Hub> for MySubscriber {
            type Emitter = MySubscriberNode;
            fn register(hub: &mut Hub, item: Rc<RefCell<Self>>) {
                hub.basic_signal.register(item);
            }
        }
        impl Named for MySubscriber {
            const NAME: &'static str = "MySubscriber";
        }
        impl BasicSignal for MySubscriber {}

        // ---

        let mut hub = Hub::new();
        let item = hub.subscribe(|_| MySubscriber);
        hub.unsubscribe(&item);
        hub.unsubscribe(&item);
    }

    #[test]
    fn double_unsubscribe_deaf_node() {
        struct Hub {
            mng: Rc<RefCell<Manager>>,
        }
        impl Hub {
            fn new() -> Self {
                let mng = Rc::new(RefCell::new(Manager::default()));
                Self { mng }
            }
        }
        impl Anchor for Hub {
            fn manager(&self) -> &Rc<RefCell<Manager>> {
                &self.mng
            }
        }

        // ---

        struct MySubscriberNode;
        impl Emit<Hub> for MySubscriberNode {
            fn create(_: &Hub) -> Self {
                Self
            }
        }
        struct MySubscriber;
        impl Subscriber<Hub> for MySubscriber {
            type Emitter = MySubscriberNode;
            fn register(_: &mut Hub, _: Rc<RefCell<Self>>) {}
        }
        impl Named for MySubscriber {
            const NAME: &'static str = "MySubscriber";
        }

        // ---

        let mut hub = Hub::new();
        let item = hub.subscribe(|_| MySubscriber);
        hub.unsubscribe(&item);
        hub.unsubscribe(&item);
    }

    #[test]
    #[should_panic(expected = "revent: name is already registered to this manager: \"signal\"")]
    fn double_subscription() {
        let mng = Rc::new(RefCell::new(Manager::default()));

        Slot::<()>::new("signal", mng.clone());
        Single::<()>::new("signal", mng);
    }

    #[test]
    #[should_panic(expected = "revent: signal modification outside of Anchor context")]
    fn using_emitter_outside_subscribe() {
        struct Hub {
            slot: Slot<()>,
            mng: Rc<RefCell<Manager>>,
        }
        impl Hub {
            fn new() -> Self {
                let mng = Rc::new(RefCell::new(Manager::default()));
                Self {
                    slot: Slot::new("slot", mng.clone()),
                    mng,
                }
            }
        }
        impl Anchor for Hub {
            fn manager(&self) -> &Rc<RefCell<Manager>> {
                &self.mng
            }
        }

        // ---

        struct Emitter;
        impl Emit<Hub> for Emitter {
            fn create(anchor: &Hub) -> Self {
                let _ = anchor.slot.clone();
                Emitter
            }
        }

        // ---

        let hub = Hub::new();
        Emitter::create(&hub);
    }

    #[test]
    #[should_panic(
        expected = "revent: not allowed to clone more than once per subscription: \"slot\""
    )]
    fn double_emit() {
        struct Hub {
            slot: Slot<()>,
            mng: Rc<RefCell<Manager>>,
        }
        impl Hub {
            fn new() -> Self {
                let mng = Rc::new(RefCell::new(Manager::default()));
                Self {
                    slot: Slot::new("slot", mng.clone()),
                    mng,
                }
            }
        }
        impl Anchor for Hub {
            fn manager(&self) -> &Rc<RefCell<Manager>> {
                &self.mng
            }
        }

        // ---

        struct Emitter;
        impl Emit<Hub> for Emitter {
            fn create(anchor: &Hub) -> Self {
                let _ = anchor.slot.clone();
                let _ = anchor.slot.clone();
                Emitter
            }
        }

        // ---

        struct Listener;
        impl Subscriber<Hub> for Listener {
            type Emitter = Emitter;
            fn register(_: &mut Hub, _: Rc<RefCell<Self>>) {}
        }
        impl Named for Listener {
            const NAME: &'static str = "Listener";
        }

        // ---

        let mut hub = Hub::new();
        hub.subscribe(|_| Listener);
    }

    #[test]
    #[should_panic(
        expected = "revent: not allowed to register more than once per subscription: \"slot\""
    )]
    fn double_register() {
        struct Hub {
            slot: Slot<Listener>,
            mng: Rc<RefCell<Manager>>,
        }
        impl Hub {
            fn new() -> Self {
                let mng = Rc::new(RefCell::new(Manager::default()));
                Self {
                    slot: Slot::new("slot", mng.clone()),
                    mng,
                }
            }
        }
        impl Anchor for Hub {
            fn manager(&self) -> &Rc<RefCell<Manager>> {
                &self.mng
            }
        }

        // ---

        struct Listener;
        impl Subscriber<Hub> for Listener {
            type Emitter = ();
            fn register(hub: &mut Hub, item: Rc<RefCell<Self>>) {
                hub.slot.register(item.clone());
                hub.slot.register(item);
            }
        }
        impl Named for Listener {
            const NAME: &'static str = "Listener";
        }

        // ---

        let mut hub = Hub::new();
        hub.subscribe(|_| Listener);
    }

    #[test]
    fn nested_subscription_same_slot() {
        struct Hub {
            slot: Slot<Listener>,
            mng: Rc<RefCell<Manager>>,
        }
        impl Hub {
            fn new() -> Self {
                let mng = Rc::new(RefCell::new(Manager::default()));
                Self {
                    slot: Slot::new("slot", mng.clone()),
                    mng,
                }
            }
        }
        impl Anchor for Hub {
            fn manager(&self) -> &Rc<RefCell<Manager>> {
                &self.mng
            }
        }

        // ---

        struct Listener {
            count: usize,
        }
        impl Subscriber<Hub> for Listener {
            type Emitter = ();
            fn register(hub: &mut Hub, item: Rc<RefCell<Self>>) {
                let count = item.borrow().count;
                if count != 0 {
                    hub.subscribe(|_| Listener { count: count - 1 });
                }
                hub.slot.register(item);
            }
        }
        impl Named for Listener {
            const NAME: &'static str = "Listener";
        }

        // ---

        let mut hub = Hub::new();
        hub.subscribe(|_| Listener { count: 100 });

        let mut count = 0;
        hub.slot.emit(|x| {
            assert_eq!(count, x.count);
            count += 1;
        });
        assert_eq!(101, count);
    }
}
