//! Asynchronous structs and functions.
pub use crossbeam_channel::RecvError;
use crossbeam_channel::{bounded, unbounded, Receiver, Sender, TryRecvError};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
enum Version {
    Bounded(usize),
    Unbounded,
}

type Senders<T> = Arc<Mutex<(Vec<Sender<T>>, Option<T>)>>;

/// Outgoing mailer. Sends a message to all associated [Mailbox]es.
///
/// Holds a list of all spawned [Mailbox]es and sends to each of these on a [send](Mailer::send).
#[derive(Clone)]
pub struct Mailer<T: Clone + Send> {
    senders: Senders<T>,
    version: Version,
}

impl<T: Clone + Send> Mailer<T> {
    /// Make a new object with bounded channels.
    pub fn bounded(capacity: usize) -> Self {
        Self {
            senders: Arc::new(Mutex::new((vec![], None))),
            version: Version::Bounded(capacity),
        }
    }

    /// Make a new object with unbounded channels.
    pub fn unbounded() -> Self {
        Self {
            senders: Arc::new(Mutex::new((vec![], None))),
            version: Version::Unbounded,
        }
    }

    /// Send an item to all receivers.
    ///
    /// Clones the item for each receiver. If this Mailer is bounded, it will block if any of
    /// the receivers are at capacity.
    pub fn send(&self, item: T) {
        let mut senders = self.senders.lock().unwrap();
        senders.0.drain_filter(|x| x.send(item.clone()).is_err());
        senders.1 = Some(item);
    }

    fn receiver(&self) -> Receiver<T> {
        let mut senders = self.senders.lock().unwrap();
        match self.version {
            Version::Bounded(count) => {
                let (tx, rx) = bounded(count);
                senders.0.push(tx);
                rx
            }
            Version::Unbounded => {
                let (tx, rx) = unbounded();
                senders.0.push(tx);
                rx
            }
        }
    }

    /// Create a receiving end corresponding to this [Mailer].
    pub fn mailbox(&self) -> Mailbox<T> {
        Mailbox {
            receiver: self.receiver(),
            senders: Arc::clone(&self.senders),
        }
    }

    /// The amount of currently active receivers.
    pub fn count(&self) -> usize {
        let senders = self.senders.lock().unwrap();
        senders.0.len()
    }
}

/// Receiving end of the [Mailer].
pub struct Mailbox<T: Clone + Send> {
    receiver: Receiver<T>,
    senders: Senders<T>,
}

impl<T: Clone + Send> Mailbox<T> {
    /// Receive a message or the last message sent. Blocks control flow.
    ///
    /// If a thread sends a message to a [Mailer] before this [Mailbox] is allocated, then
    /// this function will return the last sent message.
    pub fn recv(&self) -> T {
        match self.receiver.try_recv() {
            Ok(item) => item,
            Err(TryRecvError::Empty) => {
                let senders = self.senders.lock().unwrap();
                match &senders.1 {
                    Some(item) => item.clone(),
                    None => {
                        drop(senders);
                        match self.receiver.recv() {
                            Ok(item) => item,
                            Err(RecvError) => panic!("revent: recv: internally disconnected"),
                        }
                    }
                }
            }
            Err(TryRecvError::Disconnected) => panic!("revent: recv: internally disconnected"),
        }
    }

    /// Try receiving a message, does not block control flow.
    ///
    /// If this Mailbox was created after a message was sent, then this function will return
    /// the last message.
    ///
    /// Returns `None` if no messages were ever sent on the associated [Mailer].
    pub fn try_recv(&self) -> Option<T> {
        match self.receiver.try_recv() {
            Ok(item) => Some(item),
            Err(TryRecvError::Empty) => {
                let senders = self.senders.lock().unwrap();
                senders.1.clone()
            }
            Err(TryRecvError::Disconnected) => panic!("revent: try_recv: internally disconnected"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::asynchronous::Mailer;

    #[test]
    fn no_send_to_none() {
        let mailer: Mailer<()> = Mailer::unbounded();

        assert!(matches!(mailer.mailbox().try_recv(), None));
    }

    #[test]
    fn send_to_none() {
        let mailer = Mailer::unbounded();
        mailer.send(());

        assert!(matches!(mailer.mailbox().try_recv(), Some(())));
    }

    #[test]
    fn recv_disconnected() {
        let mailer = Mailer::unbounded();
        let mailbox = mailer.mailbox();
        mailer.send(());
        drop(mailer);

        assert!(matches!(mailbox.try_recv(), Some(())));
    }

    #[test]
    fn send_to_one() {
        let mailer = Mailer::unbounded();
        let mailbox = mailer.mailbox();
        mailer.send(());

        assert!(matches!(mailbox.try_recv(), Some(())));
    }

    #[quickcheck_macros::quickcheck]
    fn send_to_n(count: usize) {
        let mailer = Mailer::unbounded();
        let mut receivers = Vec::with_capacity(count);

        for _ in 0..count {
            receivers.push(mailer.mailbox());
        }
        mailer.send(());

        for mailbox in receivers {
            assert!(matches!(mailbox.try_recv(), Some(())));
        }
    }

    #[quickcheck_macros::quickcheck]
    fn send_to_n_with_disconnect(count: usize) {
        let mailer = Mailer::unbounded();
        let mut receivers = Vec::with_capacity(count);

        for _ in 0..count {
            receivers.push(mailer.mailbox());
        }

        assert_eq!(count, mailer.count());

        receivers.clear();

        assert_eq!(count, mailer.count());

        mailer.send(());

        assert_eq!(0, mailer.count());
    }
}
