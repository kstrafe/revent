#![doc(hidden)]
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Display, Formatter},
};

type ChannelName = &'static str;
type HandlerName = &'static str;

pub struct Manager {
    active: Vec<HandlerName>,
    amalgam: BTreeMap<ChannelName, BTreeSet<ChannelName>>,
    connections: BTreeMap<HandlerName, (BTreeSet<ChannelName>, BTreeSet<ChannelName>)>,

    emitters: BTreeMap<ChannelName, BTreeSet<HandlerName>>,
    subscribers: BTreeMap<ChannelName, BTreeSet<HandlerName>>,
}

impl Manager {
    pub fn prepare_construction(&mut self, name: &'static str) {
        self.active.push(name);
    }

    pub fn register_emit(&mut self, signal: &'static str) {
        let name = self.active.last().unwrap();
        let connection = self
            .connections
            .entry(name)
            .or_insert_with(|| Default::default());
        connection.1.insert(signal);

        self.emitters
            .entry(signal)
            .or_insert_with(Default::default)
            .insert(name);
    }

    pub fn register_subscribe(&mut self, signal: &'static str) {
        let name = self.active.last().unwrap();
        let connection = self
            .connections
            .entry(name)
            .or_insert_with(|| Default::default());
        connection.0.insert(signal);

        self.subscribers
            .entry(signal)
            .or_insert_with(Default::default)
            .insert(name);
    }

    pub fn finish_construction(&mut self) {
        let name = self.active.pop().unwrap();
        let connection = self
            .connections
            .entry(name)
            .or_insert_with(|| Default::default());
        for item in &connection.0 {
            let emit = self
                .amalgam
                .entry(item)
                .or_insert_with(|| Default::default());
            for emission in &connection.1 {
                emit.insert(emission);
            }
        }

        match chkrec(&self.amalgam) {
            Ok(()) => {}
            Err(chain) => {
                panic!(
                    "{}",
                    RecursionPrinter {
                        chain,
                        manager: self,
                    }
                );
            }
        }
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self {
            active: Default::default(),
            amalgam: Default::default(),
            connections: Default::default(),
            emitters: Default::default(),
            subscribers: Default::default(),
        }
    }
}

fn chkrec(set: &BTreeMap<ChannelName, BTreeSet<ChannelName>>) -> Result<(), Vec<ChannelName>> {
    fn chkreci(
        now: ChannelName,
        set: &BTreeMap<ChannelName, BTreeSet<ChannelName>>,
        chain: &mut Vec<ChannelName>,
    ) -> Result<(), ()> {
        if let Some(node) = set.get(&now) {
            for signal in node.iter() {
                if chain.contains(&signal) {
                    return Err(());
                }
                chain.push(*signal);
                chkreci(signal, set, chain)?;
                chain.pop();
            }
        }
        Ok(())
    }

    let mut chain = Vec::new();
    for signal in set.keys() {
        chain.push(*signal);
        if let Err(()) = chkreci(signal, set, &mut chain) {
            chain.push(chain[0]);
            return Err(chain);
        }
        chain.pop();
    }
    Ok(())
}

struct RecursionPrinter<'a> {
    chain: Vec<ChannelName>,
    manager: &'a Manager,
}

impl<'a> Display for RecursionPrinter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.chain.len() < 2 {
            panic!("internal recursion check error: chain has length < 2");
        } else if self.chain.len() >= 2 {
            for window in self.chain.windows(2) {
                let from = window[0];
                let to = window[1];

                let mut intersection = self
                    .manager
                    .subscribers
                    .get(from)
                    .unwrap()
                    .intersection(self.manager.emitters.get(to).unwrap());

                write!(f, "[")?;
                if let Some(item) = intersection.next() {
                    write!(f, "{}", item)?;
                }
                for item in intersection {
                    write!(f, ", {}", item)?;
                }
                write!(f, "]{} -> ", from)?;
            }

            write!(f, "{}", self.chain.last().unwrap())?;
        }
        Ok(())
    }
}
