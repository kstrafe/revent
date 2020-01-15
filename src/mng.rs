#![doc(hidden)]
use std::collections::{BTreeMap, BTreeSet};

/// Manage all dependencies to ensure there are no recursive events.
#[derive(Clone, Default)]
#[doc(hidden)]
pub struct Manager {
    /// Construction flag. Set by the hub when we are constructing something.
    #[doc(hidden)]
    pub construction: bool,
    subscriptions: BTreeMap<&'static str, BTreeSet<&'static str>>,

    listens: BTreeSet<&'static str>,
    emissions: BTreeSet<&'static str>,
}

impl Manager {
    /// Begin the construction of a new subscriber in the manager.
    #[doc(hidden)]
    pub fn begin_construction(&mut self) {
        self.construction = true;
    }

    /// Inform the manager that the current object under construction wishes to emit to the
    /// given channel.
    #[doc(hidden)]
    pub fn activate_channel(&mut self, name: &'static str) {
        if !self.construction {
            panic!("Activating a channel outside of construction context");
        }
        self.emissions.insert(name);
    }

    /// Inform the manager that the current object under construction wishes to subscribe to
    /// the given channel.
    #[doc(hidden)]
    pub fn subscribe_channel(&mut self, name: &'static str) {
        self.listens.insert(name);
    }

    /// End the construction of a new subscriber. Checks whether any dependency loops exist
    /// and panics if they do.
    #[doc(hidden)]
    pub fn end_construction(&mut self) {
        for from in &self.listens {
            let set = self.subscriptions.entry(from).or_insert_with(BTreeSet::new);
            for to in &self.emissions {
                set.insert(to);
            }
        }
        chkrec(&self.subscriptions);
        self.listens.clear();
        self.emissions.clear();
        self.construction = false;
    }
}

fn chkrec(set: &BTreeMap<&'static str, BTreeSet<&'static str>>) {
    fn chkreci(
        now: &'static str,
        set: &BTreeMap<&'static str, BTreeSet<&'static str>>,
        chain: &mut Vec<&'static str>,
    ) {
        if let Some(node) = set.get(&now) {
            for signal in node.iter() {
                if chain.contains(&signal) {
                    panic!("Recursion detected: {:?}", chain);
                }
                chain.push(*signal);
                chkreci(signal, set, chain);
                chain.pop();
            }
        }
    }

    let mut chain = Vec::new();
    for signal in set.keys() {
        chain.push(*signal);
        chkreci(signal, set, &mut chain);
        chain.pop();
    }
}
