#![doc(hidden)]
use std::collections::{BTreeMap, BTreeSet};

/// Manage all dependencies to ensure there are no recursive events.
#[derive(Clone)]
#[doc(hidden)]
pub struct Manager {
    subscriptions: BTreeMap<&'static str, BTreeSet<&'static str>>,

    listens: Vec<BTreeSet<&'static str>>,
    emissions: Vec<BTreeSet<&'static str>>,

    #[cfg(feature = "slog")]
    pub log: slog::Logger,
}

impl Default for Manager {
    fn default() -> Self {
        Self {
            subscriptions: Default::default(),
            listens: Default::default(),
            emissions: Default::default(),
            #[cfg(feature = "slog")]
            log: slog::Logger::root(slog::Discard, slog::o!()),
        }
    }
}

impl Manager {
    pub fn graphviz(&self) -> String {
        let mut accum = "digraph Hub {\n".to_string();
        for (k, v) in &self.subscriptions {
            accum += &format!("\t{:?} -> {:?};\n", k, v);
        }
        accum += "}";
        accum
    }

    pub fn begin_construction(&mut self) {
        self.listens.push(Default::default());
        self.emissions.push(Default::default());
    }

    pub fn activate_channel(&mut self, name: &'static str) {
        if !self.is_constructing() {
            panic!("Activating a channel outside of construction context");
        }
        self.emissions.last_mut().unwrap().insert(name);
    }

    pub fn subscribe_channel(&mut self, name: &'static str) {
        self.listens.last_mut().unwrap().insert(name);
    }

    pub fn end_construction(&mut self) {
        for from in self.listens.last().unwrap().iter() {
            let set = self.subscriptions.entry(from).or_insert_with(BTreeSet::new);
            for to in self.emissions.last().unwrap().iter() {
                set.insert(to);
            }
        }
        #[cfg(feature = "slog")]
        slog::debug!(self.log, "Object constructed";
            "listens" => format!("{:?}", self.listens.last().unwrap()),
            "emissions" => format!("{:?}", self.emissions.last().unwrap())
        );
        chkrec(&self.subscriptions);
        self.listens.pop();
        self.emissions.pop();
    }

    pub fn is_constructing(&self) -> bool {
        debug_assert_eq!(self.emissions.len(), self.listens.len());
        !self.emissions.is_empty()
    }

    #[cfg(feature = "slog")]
    pub fn emitting(&self, name: &'static str) {
        slog::trace!(self.log, "Emitting on: {}", name);
    }

    #[cfg(not(feature = "slog"))]
    pub fn emitting(&self, _: &'static str) {}
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
