#![doc(hidden)]
use std::collections::{BTreeMap, BTreeSet};

/// Manage all dependencies to ensure there are no recursive events.
#[derive(Clone)]
#[doc(hidden)]
pub struct Manager {
    /// Construction flag. Set by the hub when we are constructing something.
    #[doc(hidden)]
    pub construction: bool,
    subscriptions: BTreeMap<&'static str, BTreeSet<&'static str>>,

    listens: BTreeSet<&'static str>,
    emissions: BTreeSet<&'static str>,

    #[cfg(feature = "slog")]
    pub log: slog::Logger,
}

impl Default for Manager {
    fn default() -> Self {
        Self {
            construction: false,
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
        self.construction = true;
    }

    pub fn activate_channel(&mut self, name: &'static str) {
        if !self.construction {
            panic!("Activating a channel outside of construction context");
        }
        self.emissions.insert(name);
    }

    pub fn subscribe_channel(&mut self, name: &'static str) {
        self.listens.insert(name);
    }

    pub fn end_construction(&mut self) {
        for from in &self.listens {
            let set = self.subscriptions.entry(from).or_insert_with(BTreeSet::new);
            for to in &self.emissions {
                set.insert(to);
            }
        }
        #[cfg(feature = "slog")]
        slog::debug!(self.log, "Object constructed"; "listens" => format!("{:?}", self.listens), "emissions" => format!("{:?}", self.emissions));
        chkrec(&self.subscriptions);
        self.listens.clear();
        self.emissions.clear();
        self.construction = false;
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
