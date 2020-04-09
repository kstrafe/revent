//! Synchronous event system.
//!
//! # What is an event system? #
//!
//! An event system is a set of objects that can exchange messages with each other. The objects
//! know nothing about each other, they just know their common contracts. This allows for code to
//! be separated along logical boundaries, thus reducing complexity and increasing testability.
//!
//! In revent each such object is called a [Node](crate::Node), as it represents a node in a graph.
//! Each node decides for itself which `channel`s it wants to listen to, as well as emit to. Using
//! this information we can construct a complete graph of node-to-node interactions. As we'll see,
//! this graph is a proper directed acyclic graph (DAG), and is important for Rust's mutable aliasing
//! rules.
//!
//! Revent's events are synchronous, meaning that `emitting` an event will immediately process all
//! handlers of that event. Once the function call returns, it is guaranteed that all listeners have
//! been called.
//!
//! ## Extra ##
//!
//! As a visual aid, the documentation tests contain a [Grapher](crate::Grapher). To acquire the
//! images you'll need to run these code snippets and have `graphviz` installed.
//!
//! ## Mutable cycles ##
//!
//! Revent performs cycle detection in [subscribe](crate::Anchor::subscribe) and ensures that no
//! system exists in which we can create double mutable borrows. After subscription, no cycle
//! detection is performed, so the actual emitting and receiving part has the lowest possible
//! overhead.
//!
//! # Core Concepts #
//!
//! A revent system has 3 core concepts:
//!
//! * Channels
//! * Anchors
//! * Nodes
//!
//! ## Channel ##
//!
//! A channel is a container for item(s) that listen to that particular channel. Any signal emission on
//! said channel will notify all items in that channel.
//!
//! In this documentation `channel` denotes [Slot], [Single], as well as [feed].
//!
//! ## Anchor ##
//!
//! An anchor contains all channels in a system. It also contains a [Manager] and
//! implements [Anchor]. We register new [Node]s to an anchor, and these nodes will themselves
//! choose which channels to listen or emit to. Only anchors can be subscribed to.
//!
//! ## Nodes ##
//!
//! A node implements [Node] which contains the functions:
//!
//! * [register_emits](crate::Node::register_emits)
//! * [register_listens](crate::Node::register_listens)
//!
//! These specify the channels to emit and listen to. Any struct can be a node. Nodes are
//! constructed by first constructing the emitter structure as specified by `register_emits`. This
//! structure is then provided to the [Anchor::subscribe] `create` function.
//!
//! When talking about the nodes subscribed (as per `register_listens`) to a channel, the term `subscriber` may be used.
//!
//! # Example #
//!
//! ```
//! use revent::{Anchor, Grapher, Manager, Node, Slot};
//! use std::{cell::RefCell, rc::Rc};
//!
//! // First let's crate a hub (Anchor) that contains two signals.
//!
//! trait BasicSignal {
//!     fn basic(&mut self);
//! }
//!
//! struct MyAnchor {
//!     basic_slot_1: Slot<dyn BasicSignal>,
//!     basic_slot_2: Slot<dyn BasicSignal>,
//!     mng: Manager,
//! }
//! impl MyAnchor {
//!     fn new() -> Self {
//!         let mng = Manager::new();
//!         Self {
//!             basic_slot_1: Slot::new("basic_slot_1", &mng),
//!             basic_slot_2: Slot::new("basic_slot_2", &mng),
//!             mng,
//!         }
//!     }
//! }
//! impl Anchor for MyAnchor {
//!     fn manager(&self) -> &Manager {
//!         &self.mng
//!     }
//! }
//!
//! // ---
//!
//! // Now we define our emitter structure, this one contains only `basic_slot_2`, which indicates
//! // that we want to emit only to this slot for the subscribers using it as their register_emits.
//!
//! struct MyEmitter {
//!     basic_slot_2: Slot<dyn BasicSignal>,
//! }
//!
//! // ---
//!
//! // Create a node that uses MyEmitter (emits on `basic_slot_2`), and listens on
//! // `basic_slot_1`.
//!
//! struct MyNode { emits: MyEmitter }
//! impl Node<MyAnchor, MyEmitter> for MyNode {
//!     // Indicate which slots we want to use.
//!     fn register_emits(hub: &MyAnchor) -> MyEmitter {
//!         MyEmitter {
//!             basic_slot_2: hub.basic_slot_2.clone(),
//!         }
//!     }
//!
//!     fn register_listens(hub: &mut MyAnchor, item: Rc<RefCell<Self>>) {
//!         hub.basic_slot_1.register(item);
//!     }
//!     const NAME: &'static str = "MyNode";
//! }
//!
//! // Whenever we get a basic signal we pass it to the register_emits.
//! impl BasicSignal for MyNode {
//!     fn basic(&mut self) {
//!         self.emits.basic_slot_2.emit(|_| println!("Hello world"));
//!     }
//! }
//!
//! // ---
//!
//! let mut hub = MyAnchor::new();
//! // The type annotation is not needed, but shown here to show what to expect.
//! let item = hub.subscribe(|emits: MyEmitter| MyNode { emits });
//! hub.basic_slot_1.emit(BasicSignal::basic);
//! hub.unsubscribe(&item);
//!
//! Grapher::new(hub.manager()).graph_to_file("target/example-with-emitter.png").unwrap();
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

pub mod feed;
mod mng;
mod single;
mod slot;
mod traits;
pub(crate) use self::mng::{ChannelType, Mode};
pub use self::{
    mng::{Grapher, Manager},
    single::Single,
    slot::Slot,
    traits::{Anchor, Node},
};

use std::{cell::RefCell, rc::Rc};

thread_local! {
    static STACK: RefCell<Vec<(Mode, Manager)>> = RefCell::new(Vec::new());
}

fn assert_active_manager(manager: &Manager) {
    STACK.with(|x| {
        assert!(
            Rc::ptr_eq(
                &(x.borrow()
                    .last()
                    .expect("revent: signal modification outside of Anchor context")
                    .1)
                    .0,
                &manager.0
            ),
            "revent: manager is different"
        );
    });
}

#[cfg(test)]
mod tests {
    use crate::{
        feed::{Feed, Feedee, Feeder},
        Anchor, Manager, Node, Single, Slot,
    };
    use std::{cell::RefCell, rc::Rc};

    #[quickcheck_macros::quickcheck]
    fn basic(value: usize) {
        trait BasicSignal {}

        struct MyAnchor {
            basic_signal: Slot<dyn BasicSignal>,
            mng: Manager,
        }
        impl MyAnchor {
            fn new() -> Self {
                let mng = Manager::new();
                Self {
                    basic_signal: Slot::new("basic_signal", &mng),

                    mng,
                }
            }
        }
        impl Anchor for MyAnchor {
            fn manager(&self) -> &Manager {
                &self.mng
            }
        }

        // ---

        struct Emitter;
        struct MyNode;
        impl Node<MyAnchor, Emitter> for MyNode {
            fn register_emits(_: &MyAnchor) -> Emitter {
                Emitter
            }

            fn register_listens(hub: &mut MyAnchor, item: Rc<RefCell<Self>>) {
                hub.basic_signal.register(item);
            }
            const NAME: &'static str = "MyNode";
        }
        impl BasicSignal for MyNode {}

        // ---

        let mut hub = MyAnchor::new();

        for _ in 0..value {
            hub.subscribe(|_| MyNode);
        }

        let mut count = 0;

        hub.basic_signal.emit(|_| {
            count += 1;
        });

        assert_eq!(value, count);
    }

    #[test]
    #[should_panic(
        expected = "revent: found a recursion during subscription: [MyNode]basic_signal -> basic_signal"
    )]
    fn self_subscribing() {
        trait BasicSignal {}

        struct MyAnchor {
            basic_signal: Slot<dyn BasicSignal>,
            mng: Manager,
        }
        impl MyAnchor {
            fn new() -> Self {
                let mng = Manager::new();
                Self {
                    basic_signal: Slot::new("basic_signal", &mng),
                    mng,
                }
            }
        }
        impl Anchor for MyAnchor {
            fn manager(&self) -> &Manager {
                &self.mng
            }
        }

        // ---

        struct Emitter;
        struct MyNode;
        impl Node<MyAnchor, Emitter> for MyNode {
            fn register_emits(hub: &MyAnchor) -> Emitter {
                let _ = hub.basic_signal.clone();
                Emitter
            }
            fn register_listens(hub: &mut MyAnchor, item: Rc<RefCell<Self>>) {
                hub.basic_signal.register(item);
            }
            const NAME: &'static str = "MyNode";
        }
        impl BasicSignal for MyNode {}

        // ---

        let mut hub = MyAnchor::new();

        hub.subscribe(|_| MyNode);
    }

    #[test]
    #[should_panic(
        expected = "revent: found a recursion during subscription: [MyNode]basic_signal -> [OtherNode]other_signal -> basic_signal"
    )]
    fn transitive_self_subscription() {
        trait BasicSignal {}
        trait OtherSignal {}

        struct MyAnchor {
            basic_signal: Slot<dyn BasicSignal>,
            other_signal: Slot<dyn OtherSignal>,
            mng: Manager,
        }
        impl MyAnchor {
            fn new() -> Self {
                let mng = Manager::new();
                Self {
                    basic_signal: Slot::new("basic_signal", &mng),
                    other_signal: Slot::new("other_signal", &mng),
                    mng,
                }
            }
        }
        impl Anchor for MyAnchor {
            fn manager(&self) -> &Manager {
                &self.mng
            }
        }

        // ---

        struct Emitter;
        struct MyNode;
        impl Node<MyAnchor, Emitter> for MyNode {
            fn register_emits(hub: &MyAnchor) -> Emitter {
                let _ = hub.other_signal.clone();
                Emitter
            }
            fn register_listens(hub: &mut MyAnchor, item: Rc<RefCell<Self>>) {
                hub.basic_signal.register(item);
            }
            const NAME: &'static str = "MyNode";
        }
        impl BasicSignal for MyNode {}

        // ---

        struct OtherNodeNode;
        struct OtherNode;
        impl Node<MyAnchor, OtherNodeNode> for OtherNode {
            fn register_emits(hub: &MyAnchor) -> OtherNodeNode {
                let _ = hub.basic_signal.clone();
                OtherNodeNode
            }
            fn register_listens(hub: &mut MyAnchor, item: Rc<RefCell<Self>>) {
                hub.other_signal.register(item);
            }
            const NAME: &'static str = "OtherNode";
        }
        impl OtherSignal for OtherNode {}

        // ---

        let mut hub = MyAnchor::new();

        hub.subscribe(|_| MyNode);
        hub.subscribe(|_| OtherNode);
    }

    #[quickcheck_macros::quickcheck]
    fn register_listens_and_unsubscribe(subscribes: usize) {
        trait BasicSignal {}

        struct MyAnchor {
            basic_signal: Slot<dyn BasicSignal>,
            mng: Manager,
        }
        impl MyAnchor {
            fn new() -> Self {
                let mng = Manager::new();
                Self {
                    basic_signal: Slot::new("basic_signal", &mng),
                    mng,
                }
            }
        }
        impl Anchor for MyAnchor {
            fn manager(&self) -> &Manager {
                &self.mng
            }
        }

        // ---

        struct Emitter;
        struct MyNode;
        impl Node<MyAnchor, Emitter> for MyNode {
            fn register_emits(_: &MyAnchor) -> Emitter {
                Emitter
            }
            fn register_listens(hub: &mut MyAnchor, item: Rc<RefCell<Self>>) {
                hub.basic_signal.register(item);
            }
            const NAME: &'static str = "MyNode";
        }
        impl BasicSignal for MyNode {}

        // ---

        let mut hub = MyAnchor::new();

        let mut items = Vec::with_capacity(subscribes);
        for _ in 0..subscribes {
            items.push(hub.subscribe(|_| MyNode));
        }

        {
            let mut count = 0;
            hub.basic_signal.emit(|_| {
                count += 1;
            });
            assert_eq!(subscribes, count);
        }

        for item in items.drain(..) {
            hub.unsubscribe(&item);
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

        struct MyAnchor {
            basic_signal: Slot<dyn BasicSignal>,
            mng: Manager,
        }
        impl MyAnchor {
            fn new() -> Self {
                let mng = Manager::new();
                Self {
                    basic_signal: Slot::new("basic_signal", &mng),
                    mng,
                }
            }
        }
        impl Anchor for MyAnchor {
            fn manager(&self) -> &Manager {
                &self.mng
            }
        }

        // ---

        struct Emitter;
        struct MyNode;
        impl Node<MyAnchor, Emitter> for MyNode {
            fn register_emits(_: &MyAnchor) -> Emitter {
                Emitter
            }
            fn register_listens(hub: &mut MyAnchor, item: Rc<RefCell<Self>>) {
                hub.basic_signal.register(item);
            }
            const NAME: &'static str = "MyNode";
        }
        impl BasicSignal for MyNode {}

        // ---

        let mut hub = MyAnchor::new();
        let item = hub.subscribe(|_| MyNode);
        hub.unsubscribe(&item);
        hub.unsubscribe(&item);
    }

    #[test]
    fn double_unsubscribe_deaf_node() {
        struct MyAnchor {
            mng: Manager,
        }
        impl MyAnchor {
            fn new() -> Self {
                let mng = Manager::new();
                Self { mng }
            }
        }
        impl Anchor for MyAnchor {
            fn manager(&self) -> &Manager {
                &self.mng
            }
        }

        // ---

        struct Emitter;
        struct MyNode;
        impl Node<MyAnchor, Emitter> for MyNode {
            fn register_emits(_: &MyAnchor) -> Emitter {
                Emitter
            }
            fn register_listens(_: &mut MyAnchor, _: Rc<RefCell<Self>>) {}
            const NAME: &'static str = "MyNode";
        }

        // ---

        let mut hub = MyAnchor::new();
        let item = hub.subscribe(|_| MyNode);
        hub.unsubscribe(&item);
        hub.unsubscribe(&item);
    }

    #[test]
    #[should_panic(expected = "revent: name is already registered to this manager: \"signal\"")]
    fn double_subscription() {
        let mng = Manager::new();

        Slot::<()>::new("signal", &mng);
        Single::<()>::new("signal", &mng);
    }

    #[test]
    #[should_panic(expected = "revent: signal modification outside of Anchor context")]
    fn using_register_emits_outside_subscribe() {
        struct MyAnchor {
            slot: Slot<()>,
            mng: Manager,
        }
        impl MyAnchor {
            fn new() -> Self {
                let mng = Manager::new();
                Self {
                    slot: Slot::new("slot", &mng),
                    mng,
                }
            }
        }
        impl Anchor for MyAnchor {
            fn manager(&self) -> &Manager {
                &self.mng
            }
        }

        // ---

        let hub = MyAnchor::new();
        let _ = hub.slot.clone();
    }

    #[test]
    #[should_panic(
        expected = "revent: not allowed to clone more than once per subscription: \"slot\""
    )]
    fn double_emit() {
        struct MyAnchor {
            slot: Slot<()>,
            mng: Manager,
        }
        impl MyAnchor {
            fn new() -> Self {
                let mng = Manager::new();
                Self {
                    slot: Slot::new("slot", &mng),
                    mng,
                }
            }
        }
        impl Anchor for MyAnchor {
            fn manager(&self) -> &Manager {
                &self.mng
            }
        }

        // ---

        struct Emitter;

        // ---

        struct Listener;
        impl Node<MyAnchor, Emitter> for Listener {
            fn register_emits(hub: &MyAnchor) -> Emitter {
                let _ = hub.slot.clone();
                let _ = hub.slot.clone();
                Emitter
            }
            fn register_listens(_: &mut MyAnchor, _: Rc<RefCell<Self>>) {}
            const NAME: &'static str = "Listener";
        }

        // ---

        let mut hub = MyAnchor::new();
        hub.subscribe(|_| Listener);
    }

    #[test]
    #[should_panic(
        expected = "revent: not allowed to register more than once per subscription: \"slot\""
    )]
    fn double_register() {
        struct MyAnchor {
            slot: Slot<Listener>,
            mng: Manager,
        }
        impl MyAnchor {
            fn new() -> Self {
                let mng = Manager::new();
                Self {
                    slot: Slot::new("slot", &mng),
                    mng,
                }
            }
        }
        impl Anchor for MyAnchor {
            fn manager(&self) -> &Manager {
                &self.mng
            }
        }

        // ---

        struct Listener;
        impl Node<MyAnchor, ()> for Listener {
            fn register_emits(_: &MyAnchor) -> () {
                ()
            }
            fn register_listens(hub: &mut MyAnchor, item: Rc<RefCell<Self>>) {
                hub.slot.register(item.clone());
                hub.slot.register(item);
            }
            const NAME: &'static str = "Listener";
        }

        // ---

        let mut hub = MyAnchor::new();
        hub.subscribe(|_| Listener);
    }

    #[test]
    fn nested_subscription_same_slot() {
        struct MyAnchor {
            slot: Slot<Listener>,
            mng: Manager,
        }
        impl MyAnchor {
            fn new() -> Self {
                let mng = Manager::new();
                Self {
                    slot: Slot::new("slot", &mng),
                    mng,
                }
            }
        }
        impl Anchor for MyAnchor {
            fn manager(&self) -> &Manager {
                &self.mng
            }
        }

        // ---

        struct Listener {
            count: usize,
        }
        impl Node<MyAnchor, ()> for Listener {
            fn register_emits(_: &MyAnchor) -> () {
                ()
            }
            fn register_listens(hub: &mut MyAnchor, item: Rc<RefCell<Self>>) {
                let count = item.borrow().count;
                if count != 0 {
                    hub.subscribe(|_| Listener { count: count - 1 });
                }
                hub.slot.register(item);
            }
            const NAME: &'static str = "Listener";
        }

        // ---

        let mut hub = MyAnchor::new();
        hub.subscribe(|_| Listener { count: 100 });

        let mut count = 0;
        hub.slot.emit(|x| {
            assert_eq!(count, x.count);
            count += 1;
        });
        assert_eq!(101, count);
    }

    #[test]
    fn using_queues() {
        struct MyAnchor {
            queue: Feed<usize>,
            mng: Manager,
        }
        impl MyAnchor {
            fn new() -> Self {
                let mng = Manager::new();
                Self {
                    queue: Feed::new("queue", &mng, 1),
                    mng,
                }
            }
        }
        impl Anchor for MyAnchor {
            fn manager(&self) -> &Manager {
                &self.mng
            }
        }

        // ---

        struct FeederNode;
        impl Node<MyAnchor, ()> for FeederNode {
            fn register_emits(anchor: &MyAnchor) {
                anchor.queue.feeder().feed(0);
            }
            fn register_listens(_: &mut MyAnchor, _: Rc<RefCell<Self>>) {}
            const NAME: &'static str = "FeederNode";
        }

        // ---

        struct FeedeeNode {
            queue: Feedee<usize>,
        }
        impl Node<MyAnchor, Feedee<usize>> for FeedeeNode {
            fn register_emits(anchor: &MyAnchor) -> Feedee<usize> {
                anchor.queue.feedee()
            }
            fn register_listens(_: &mut MyAnchor, _: Rc<RefCell<Self>>) {}
            const NAME: &'static str = "FeedeeNode";
        }

        impl FeedeeNode {
            fn pop(&mut self) -> Option<usize> {
                self.queue.pop()
            }
        }

        // ---

        let mut hub = MyAnchor::new();
        let feedee = hub.subscribe(|queue| FeedeeNode { queue });
        hub.subscribe(|_| FeederNode);

        assert_eq!(Some(0), feedee.borrow_mut().pop());
    }

    #[test]
    #[should_panic(expected = "revent: name is already registered to this manager: \"lorem\"")]
    fn double_receiver() {
        let mng = Manager::new();

        Feed::<()>::new("lorem", &mng, 1);
        Slot::<()>::new("lorem", &mng);
    }

    #[test]
    fn recur_via_feed() {
        struct MyAnchor {
            feed: Feed<()>,
            mng: Manager,
        }
        impl MyAnchor {
            fn new() -> Self {
                let mng = Manager::new();
                Self {
                    feed: Feed::new("queue", &mng, 1),
                    mng,
                }
            }
        }
        impl Anchor for MyAnchor {
            fn manager(&self) -> &Manager {
                &self.mng
            }
        }

        // ---

        struct MyNode;
        impl Node<MyAnchor, ()> for MyNode {
            fn register_emits(anchor: &MyAnchor) {
                let _ = anchor.feed.feeder();
                let _ = anchor.feed.feedee();
            }
            fn register_listens(_: &mut MyAnchor, _: Rc<RefCell<Self>>) {}
            const NAME: &'static str = "MyNode";
        }

        // ---

        let mut hub = MyAnchor::new();
        hub.subscribe(|_| MyNode);
    }

    #[test]
    fn one_feeder_to_many_feedees() {
        struct MyAnchor {
            queue: Feed<usize>,
            mng: Manager,
        }
        impl MyAnchor {
            fn new() -> Self {
                let mng = Manager::new();
                Self {
                    queue: Feed::new("queue", &mng, 2),
                    mng,
                }
            }
        }
        impl Anchor for MyAnchor {
            fn manager(&self) -> &Manager {
                &self.mng
            }
        }

        // ---

        struct FeederNode;
        impl Node<MyAnchor, ()> for FeederNode {
            fn register_emits(anchor: &MyAnchor) {
                anchor.queue.feeder().feed(0);
            }
            fn register_listens(_: &mut MyAnchor, _: Rc<RefCell<Self>>) {}
            const NAME: &'static str = "FeederNode";
        }

        // ---

        struct FeedeeNode {
            queue: Feedee<usize>,
        }
        impl Node<MyAnchor, Feedee<usize>> for FeedeeNode {
            fn register_emits(anchor: &MyAnchor) -> Feedee<usize> {
                anchor.queue.feedee()
            }
            fn register_listens(_: &mut MyAnchor, _: Rc<RefCell<Self>>) {}
            const NAME: &'static str = "FeedeeNode";
        }

        impl FeedeeNode {
            fn pop(&mut self) -> Option<usize> {
                self.queue.pop()
            }
        }

        // ---

        let mut hub = MyAnchor::new();
        let mut subs = Vec::new();
        for _ in 0..100 {
            subs.push(hub.subscribe(|queue| FeedeeNode { queue }));
        }
        hub.subscribe(|_| FeederNode);

        for sub in &subs {
            assert_eq!(Some(0), sub.borrow_mut().pop());
            assert_eq!(None, sub.borrow_mut().pop());
        }
        hub.subscribe(|_| FeederNode);
        hub.subscribe(|_| FeederNode);
        for sub in subs {
            assert_eq!(Some(0), sub.borrow_mut().pop());
            assert_eq!(Some(0), sub.borrow_mut().pop());
            assert_eq!(None, sub.borrow_mut().pop());
        }
    }

    #[test]
    #[should_panic(
        expected = "revent: feedee queue exceeds maximum size: 1, feedee: \"FeedeeNode\""
    )]
    fn feeder_channel_limit_single() {
        struct MyAnchor {
            queue: Feed<usize>,
            mng: Manager,
        }
        impl MyAnchor {
            fn new() -> Self {
                let mng = Manager::new();
                Self {
                    queue: Feed::new("queue", &mng, 1),
                    mng,
                }
            }
        }
        impl Anchor for MyAnchor {
            fn manager(&self) -> &Manager {
                &self.mng
            }
        }

        // ---

        struct FeederNode;
        impl Node<MyAnchor, ()> for FeederNode {
            fn register_emits(anchor: &MyAnchor) {
                anchor.queue.feeder().feed(0);
            }
            fn register_listens(_: &mut MyAnchor, _: Rc<RefCell<Self>>) {}
            const NAME: &'static str = "FeederNode";
        }

        // ---

        struct FeedeeNode {
            _queue: Feedee<usize>,
        }
        impl Node<MyAnchor, Feedee<usize>> for FeedeeNode {
            fn register_emits(anchor: &MyAnchor) -> Feedee<usize> {
                anchor.queue.feedee()
            }
            fn register_listens(_: &mut MyAnchor, _: Rc<RefCell<Self>>) {}
            const NAME: &'static str = "FeedeeNode";
        }

        // ---

        let mut hub = MyAnchor::new();
        let _feedee = hub.subscribe(|queue| FeedeeNode { _queue: queue });
        hub.subscribe(|_| FeederNode);
        hub.subscribe(|_| FeederNode);
    }

    #[test]
    #[should_panic(
        expected = "revent: feedee queue exceeds maximum size: 1, feedee: \"FeedeeNode\""
    )]
    fn feeder_channel_limit() {
        struct MyAnchor {
            queue: Feed<usize>,
            mng: Manager,
        }
        impl MyAnchor {
            fn new() -> Self {
                let mng = Manager::new();
                Self {
                    queue: Feed::new("queue", &mng, 1),
                    mng,
                }
            }
        }
        impl Anchor for MyAnchor {
            fn manager(&self) -> &Manager {
                &self.mng
            }
        }

        // ---

        struct FeederNode;
        impl Node<MyAnchor, ()> for FeederNode {
            fn register_emits(anchor: &MyAnchor) {
                anchor.queue.feeder().feed(0);
            }
            fn register_listens(_: &mut MyAnchor, _: Rc<RefCell<Self>>) {}
            const NAME: &'static str = "FeederNode";
        }

        // ---

        struct FeedeeNode {
            queue: Feedee<usize>,
        }
        impl Node<MyAnchor, Feedee<usize>> for FeedeeNode {
            fn register_emits(anchor: &MyAnchor) -> Feedee<usize> {
                anchor.queue.feedee()
            }
            fn register_listens(_: &mut MyAnchor, _: Rc<RefCell<Self>>) {}
            const NAME: &'static str = "FeedeeNode";
        }

        impl FeedeeNode {
            fn pop(&mut self) -> Option<usize> {
                self.queue.pop()
            }
        }

        // ---

        let mut hub = MyAnchor::new();
        let mut subs = Vec::new();
        for _ in 0..100 {
            subs.push(hub.subscribe(|queue| FeedeeNode { queue }));
        }
        hub.subscribe(|_| FeederNode);

        for sub in &subs {
            assert_eq!(Some(0), sub.borrow_mut().pop());
            assert_eq!(None, sub.borrow_mut().pop());
        }
        hub.subscribe(|_| FeederNode);
        hub.subscribe(|_| FeederNode);
        for sub in subs {
            assert_eq!(Some(0), sub.borrow_mut().pop());
            assert_eq!(Some(0), sub.borrow_mut().pop());
            assert_eq!(None, sub.borrow_mut().pop());
        }
    }

    #[quickcheck_macros::quickcheck]
    fn feed_size_checks(max_size: usize) {
        struct MyAnchor {
            queue: Feed<()>,
            mng: Manager,
        }
        impl MyAnchor {
            fn new(max_size: usize) -> Self {
                let mng = Manager::new();
                Self {
                    queue: Feed::new("queue", &mng, max_size),
                    mng,
                }
            }
        }
        impl Anchor for MyAnchor {
            fn manager(&self) -> &Manager {
                &self.mng
            }
        }

        // ---

        struct FeederNode {
            queue: Feeder<()>,
        }
        impl Node<MyAnchor, Feeder<()>> for FeederNode {
            fn register_emits(anchor: &MyAnchor) -> Feeder<()> {
                anchor.queue.feeder()
            }
            fn register_listens(_: &mut MyAnchor, _: Rc<RefCell<Self>>) {}
            const NAME: &'static str = "FeederNode";
        }
        impl FeederNode {
            fn push(&mut self) {
                self.queue.feed(());
            }
        }

        // ---

        struct FeedeeNode {
            _queue: Feedee<()>,
        }
        impl Node<MyAnchor, Feedee<()>> for FeedeeNode {
            fn register_emits(anchor: &MyAnchor) -> Feedee<()> {
                anchor.queue.feedee()
            }
            fn register_listens(_: &mut MyAnchor, _: Rc<RefCell<Self>>) {}
            const NAME: &'static str = "FeedeeNode";
        }

        // ---

        let mut hub = MyAnchor::new(max_size);
        let _feedee = hub.subscribe(|queue| FeedeeNode { _queue: queue });
        let feeder = hub.subscribe(|queue| FeederNode { queue });

        for _ in 0..max_size {
            feeder.borrow_mut().push();
        }
    }

    #[test]
    fn one_feeder_to_many_feedees_disable_enable() {
        struct MyAnchor {
            queue: Feed<usize>,
            mng: Manager,
        }
        impl MyAnchor {
            fn new() -> Self {
                let mng = Manager::new();
                Self {
                    queue: Feed::new("queue", &mng, 2),
                    mng,
                }
            }
        }
        impl Anchor for MyAnchor {
            fn manager(&self) -> &Manager {
                &self.mng
            }
        }

        // ---

        struct FeederNode;
        impl Node<MyAnchor, ()> for FeederNode {
            fn register_emits(anchor: &MyAnchor) {
                anchor.queue.feeder().feed(0);
            }
            fn register_listens(_: &mut MyAnchor, _: Rc<RefCell<Self>>) {}
            const NAME: &'static str = "FeederNode";
        }

        // ---

        struct FeedeeNode {
            queue: Feedee<usize>,
        }
        impl Node<MyAnchor, Feedee<usize>> for FeedeeNode {
            fn register_emits(anchor: &MyAnchor) -> Feedee<usize> {
                anchor.queue.feedee()
            }
            fn register_listens(_: &mut MyAnchor, _: Rc<RefCell<Self>>) {}
            const NAME: &'static str = "FeedeeNode";
        }

        impl FeedeeNode {
            fn new(mut queue: Feedee<usize>, enable: bool) -> Self {
                if !enable {
                    queue.disable();
                }
                Self { queue }
            }
            fn pop(&mut self) -> Option<usize> {
                self.queue.pop()
            }
        }

        // ---

        let mut hub = MyAnchor::new();
        let sub_1 = hub.subscribe(|queue| FeedeeNode::new(queue, true));
        let sub_2 = hub.subscribe(|queue| FeedeeNode::new(queue, false));

        hub.subscribe(|_| FeederNode);

        assert_eq!(Some(0), sub_1.borrow_mut().pop());
        assert_eq!(None, sub_2.borrow_mut().pop());
    }
}
