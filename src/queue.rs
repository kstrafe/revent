use crate::{assert_active_manager, Manager};
use std::{cell::RefCell, mem::replace, rc::Rc};

/// Receiver slot. A slot that stores specific messages.
pub struct Receiver<T> {
    manager: Rc<RefCell<Manager>>,
    nodes: Rc<RefCell<Vec<T>>>,
}

impl<T> Receiver<T> {
    /// Create a new receiver object.
    pub fn new(name: &'static str, manager: Rc<RefCell<Manager>>) -> Self {
        manager.borrow_mut().ensure_queue(name);
        Self {
            manager,
            nodes: Rc::new(RefCell::new(Vec::new())),
        }
    }

    /// Get a sender to this receiver. Only valid in an [Anchor::subscribe](crate::Anchor::subscribe) context.
    pub fn sender(&self) -> Sender<T> {
        assert_active_manager(&self.manager);
        Sender {
            nodes: self.nodes.clone(),
        }
    }

    /// Extract the received messages by replacing the queue with another.
    ///
    /// Received messages are in the order they were pushed by [Sender::push].
    ///
    /// # Performance #
    ///
    /// It is best practice to reuse the same 2 vector objects here, since [Vec] does not shrink to
    /// fit, after some time these vectors will become large enough to never need resizing. This is
    /// a performance feature.
    pub fn exchange(&mut self, vector: Vec<T>) -> Vec<T> {
        replace(&mut *self.nodes.borrow_mut(), vector)
    }
}

/// Counterpart to [Receiver]. To create one see [Receiver::sender].
pub struct Sender<T> {
    nodes: Rc<RefCell<Vec<T>>>,
}

impl<T> Sender<T> {
    /// Push data to this queue.
    pub fn push(&mut self, item: T) {
        self.nodes.borrow_mut().push(item);
    }
}

#[cfg(test)]
mod tests {
    use crate::{Manager, Receiver};
    use std::{cell::RefCell, rc::Rc};

    #[test]
    #[should_panic(expected = "revent: name is already registered to this manager: \"receiver\"")]
    fn double_receiver() {
        let mng = Rc::new(RefCell::new(Manager::new()));

        Receiver::<()>::new("receiver", mng.clone());
        Receiver::<()>::new("receiver", mng);
    }
}
