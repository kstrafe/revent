//! Cycle-breaking poll-based backward channels.
//!
//! A `feed` is a "feedback" from a "bottom-node" (a node that can not send any further signals as
//! that would cause cycles) to any of its ascendants. The parents will have to pop elements from the
//! feed whenever they want, which is typically done right after calling slot(s).
//!
//! A feed consumer [Node](crate::Node) (called a [Feedee]) will have its own unique queue. This means that two
//! nodes can use the same [Feeder] source, and consume objects at different rates without
//! affecting each other. Items sent from the feeder are cloned to all feedees.
//!
//! ```
//! use revent::{Anchor, feed::{Feed, Feedee, Feeder}, Grapher, Manager, Node, Slot};
//! use std::{cell::RefCell, rc::Rc};
//!
//! trait BasicSignal {
//!     fn basic(&mut self);
//! }
//!
//! struct MyAnchor {
//!     basic_slot_1: Slot<dyn BasicSignal>,
//!     basic_slot_2: Slot<dyn BasicSignal>,
//!     feedback: Feed<usize>,
//!     mng: Manager,
//! }
//! impl MyAnchor {
//!     fn new() -> Self {
//!         let mng = Manager::new();
//!         Self {
//!             basic_slot_1: Slot::new("basic_slot_1", &mng),
//!             basic_slot_2: Slot::new("basic_slot_2", &mng),
//!             feedback: Feed::new("feedback", &mng),
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
//! struct EmitterA {
//!     basic_slot_2: Slot<dyn BasicSignal>,
//!     feedback: Feedee<usize>,
//! }
//! struct A { emits: EmitterA }
//! impl Node<MyAnchor, EmitterA> for A {
//!     fn register_emits(hub: &MyAnchor) -> EmitterA {
//!         EmitterA {
//!             basic_slot_2: hub.basic_slot_2.clone(),
//!             feedback: hub.feedback.feedee(),
//!         }
//!     }
//!
//!     fn register_listens(hub: &mut MyAnchor, item: Rc<RefCell<Self>>) {
//!         hub.basic_slot_1.register(item);
//!     }
//!     const NAME: &'static str = "A";
//! }
//!
//! // Whenever we get a basic signal we pass it to the register_emits.
//! impl BasicSignal for A {
//!     fn basic(&mut self) {
//!         self.emits.basic_slot_2.emit(BasicSignal::basic);
//!         while let Some(item) = self.emits.feedback.pop() {
//!             println!("A: Got feedback: {}", item);
//!         }
//!     }
//! }
//!
//! // ---
//!
//! struct EmitterB {
//!     feedback: Feeder<usize>,
//! }
//! struct B { emits: EmitterB }
//! impl Node<MyAnchor, EmitterB> for B {
//!     fn register_emits(hub: &MyAnchor) -> EmitterB {
//!         EmitterB {
//!             feedback: hub.feedback.feeder(),
//!         }
//!     }
//!
//!     fn register_listens(hub: &mut MyAnchor, item: Rc<RefCell<Self>>) {
//!         hub.basic_slot_2.register(item);
//!     }
//!     const NAME: &'static str = "B";
//! }
//!
//! // Whenever we get a basic signal we pass it to the register_emits.
//! impl BasicSignal for B {
//!     fn basic(&mut self) {
//!         println!("Node B: Sending feedback to all subscribers");
//!         self.emits.feedback.feed(123);
//!     }
//! }
//!
//! // ---
//!
//! let mut hub = MyAnchor::new();
//! hub.subscribe(|emits| A { emits });
//! hub.subscribe(|emits| B { emits });
//!
//! Grapher::new(hub.manager()).graph_to_file("target/feeds.png").unwrap();
//! ```
use crate::{assert_active_manager, ChannelType, Manager};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};

type Queue<T> = Rc<RefCell<VecDeque<T>>>;

/// Sender part of [Feed].
pub struct Feeder<T: Clone> {
    queues: Rc<RefCell<Vec<Queue<T>>>>,
}

impl<T: Clone> Feeder<T> {
    /// Push an item to this queue.
    ///
    /// All [Feedee]s associated with this feeder will have the input `item` pushed onto their
    /// queues.
    pub fn feed(&self, item: T) {
        let mut queues = self.queues.borrow_mut();
        if let Some((last, rest)) = queues.split_last_mut() {
            for queue in rest.iter_mut() {
                queue.borrow_mut().push_back(item.clone());
            }
            last.borrow_mut().push_back(item);
        }
    }

    // /// Optimization of [feed](Feeder::feed) for small objects.
    // pub fn feed_small(&self, item: T) {
    //     for queue in self.queues.borrow_mut().iter_mut() {
    //         queue.borrow_mut().push_back(item.clone());
    //     }
    // }
}

/// Receiver part of [Feed].
pub struct Feedee<T> {
    queues: Rc<RefCell<Vec<Queue<T>>>>,
    queue: Queue<T>,
}

impl<T> Feedee<T> {
    /// Get an item from the front of the queue.
    pub fn pop(&mut self) -> Option<T> {
        self.queue.borrow_mut().pop_front()
    }
}

impl<T> Drop for Feedee<T> {
    fn drop(&mut self) {
        self.queues
            .borrow_mut()
            .retain(|item| !Rc::ptr_eq(item, &self.queue));
    }
}

/// Feedback mechanism to provide data to [Node](crate::Node)s higher up in the revent DAG.
pub struct Feed<T> {
    manager: Manager,
    name: &'static str,
    queues: Rc<RefCell<Vec<Queue<T>>>>,
}

impl<T: Clone> Feed<T> {
    /// Create a new feed.
    pub fn new(name: &'static str, manager: &Manager) -> Self {
        manager.ensure_new(name, ChannelType::Feed);
        Self {
            manager: manager.clone(),
            name,
            queues: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Create a feed sender.
    pub fn feeder(&self) -> Feeder<T> {
        assert_active_manager(&self.manager);
        self.manager.register_emit(self.name);
        Feeder {
            queues: self.queues.clone(),
        }
    }

    /// Create a feed receiver.
    ///
    /// Each receiver has its own internal queue. Sending a message via a feeder while 2 feedees
    /// exist will duplicate the message to both feedees. The feedees do not interfere with each
    /// other.
    pub fn feedee(&self) -> Feedee<T> {
        assert_active_manager(&self.manager);
        self.manager.register_subscribe(self.name);
        let queue = Rc::new(RefCell::new(VecDeque::new()));
        self.queues.borrow_mut().push(queue.clone());
        Feedee {
            queues: self.queues.clone(),
            queue,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{feed::Feed, Manager};

    #[test]
    #[should_panic(expected = "revent: name is already registered to this manager: \"feed\"")]
    fn double_receiver() {
        let mng = Manager::new();

        Feed::<()>::new("feed", &mng);
        Feed::<()>::new("feed", &mng);
    }
}
