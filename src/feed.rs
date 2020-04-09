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
//!             feedback: Feed::new("feedback", &mng, 1),
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

struct Queue<T> {
    items: Rc<RefCell<VecDeque<T>>>,
    name: &'static str,
}

impl<T> Clone for Queue<T> {
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            name: self.name,
        }
    }
}

/// Sender part of [Feed].
pub struct Feeder<T: Clone> {
    max_size: usize,
    queues: Rc<RefCell<Vec<Queue<T>>>>,
}

impl<T: Clone> Feeder<T> {
    /// Push an item to this queue.
    ///
    /// All [Feedee]s associated with this feeder will have the input `item` pushed onto their
    /// queues.
    ///
    /// # Panics #
    ///
    /// Panics if the queue for a [Feedee] is full.
    pub fn feed(&self, item: T) {
        let mut queues = self.queues.borrow_mut();
        if let Some((last, rest)) = queues.split_last_mut() {
            for queue in rest.iter_mut() {
                let (mut queue, name) = (queue.items.borrow_mut(), queue.name);
                if queue.len() == self.max_size {
                    panic!(
                        "revent: feedee queue exceeds maximum size: {}, feedee: {}",
                        self.max_size, name,
                    );
                }
                queue.push_back(item.clone());
            }

            let (mut queue, name) = (last.items.borrow_mut(), last.name);
            if queue.len() == self.max_size {
                panic!(
                    "revent: feedee queue exceeds maximum size: {}, feedee: {}",
                    self.max_size, name,
                );
            }
            queue.push_back(item);
        }
    }
}

/// Receiver part of [Feed].
pub struct Feedee<T> {
    queues: Rc<RefCell<Vec<Queue<T>>>>,
    queue: Queue<T>,
}

impl<T> Feedee<T> {
    /// Get an item from the front of the queue.
    pub fn pop(&mut self) -> Option<T> {
        self.queue.items.borrow_mut().pop_front()
    }

    /// Enable this receiver.
    ///
    /// Feedees are enabled by default.
    ///
    /// This function is idempotent, meaning that calling it multiple times has no effect if
    /// the feedee is already enabled.
    ///
    /// # Returns #
    ///
    /// True if the state changed from disabled to enabled. False otherwise.
    pub fn enable(&mut self) -> bool {
        let mut queues = self.queues.borrow_mut();

        let len_before = queues.len();
        queues.retain(|item| !Rc::ptr_eq(&item.items, &self.queue.items));
        queues.push(self.queue.clone());
        let len_after = queues.len();

        len_before != len_after
    }

    /// Disable this receiver. The [Feeder] will not be able to push data to this queue.
    ///
    /// This function is idempotent, meaning that calling it multiple times has no effect if
    /// the feedee is already disabled.
    ///
    /// # Returns #
    ///
    /// True if the state changed from enabled to disabled. False otherwise.
    pub fn disable(&mut self) -> bool {
        let mut queues = self.queues.borrow_mut();
        let len_before = queues.len();
        queues.retain(|item| !Rc::ptr_eq(&item.items, &self.queue.items));
        let len_after = queues.len();

        len_before != len_after
    }
}

impl<T> Drop for Feedee<T> {
    fn drop(&mut self) {
        self.queues
            .borrow_mut()
            .retain(|item| !Rc::ptr_eq(&item.items, &self.queue.items));
    }
}

/// Feedback mechanism to provide data to [Node](crate::Node)s higher up in the revent DAG.
pub struct Feed<T> {
    manager: Manager,
    max_size: usize,
    name: &'static str,
    queues: Rc<RefCell<Vec<Queue<T>>>>,
}

impl<T: Clone> Feed<T> {
    /// Create a new feed.
    pub fn new(name: &'static str, manager: &Manager, max_size: usize) -> Self {
        manager.ensure_new(name, ChannelType::Feed);
        Self {
            manager: manager.clone(),
            max_size,
            name,
            queues: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Create a feed sender.
    pub fn feeder(&self) -> Feeder<T> {
        assert_active_manager(&self.manager);
        self.manager.register_emit(self.name);
        Feeder {
            max_size: self.max_size,
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
        self.manager.register_listen(self.name);
        let queue = Queue {
            items: Rc::new(RefCell::new(VecDeque::new())),
            name: self.manager.current(),
        };
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

        Feed::<()>::new("feed", &mng, 1);
        Feed::<()>::new("feed", &mng, 1);
    }
}
