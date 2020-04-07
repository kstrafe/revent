use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Debug, Display, Formatter},
};

type ChannelName = &'static str;
type HandlerName = &'static str;

#[derive(Clone, Copy)]
pub(crate) enum Mode {
    Adding,
    Removing,
}

#[derive(Debug)]
struct ListensAndEmits {
    name: HandlerName,
    emits: Vec<ChannelName>,
    listens: Vec<ChannelName>,
}

/// Inspects how various [Node](crate::Node)s use slots.
///
/// Will [panic] if there exists any subscriber cycle. Cycle detection occurs only during
/// [Anchor::subscribe](crate::Anchor::subscribe). Emitting will not perform any cycle detection.
///
/// Unsubscribing items does not remove the channel dependencies from the manager. This is
/// intentional to discourage juggling subscriptions to fit the dependency chain.
#[derive(Debug)]
pub struct Manager {
    active: Vec<ListensAndEmits>,
    amalgam: BTreeMap<ChannelName, BTreeSet<ChannelName>>,

    emits: BTreeMap<ChannelName, BTreeSet<HandlerName>>,
    listens: BTreeMap<ChannelName, BTreeSet<HandlerName>>,

    queues: BTreeSet<ChannelName>,
}

impl Manager {
    /// Create a new manager.
    pub fn new() -> Self {
        Self::default()
    }

    fn unique_name(&self, name: &'static str) {
        assert!(
            !self.queues.contains(name) && !self.amalgam.contains_key(name),
            "revent: name is already registered to this manager: {:?}",
            name
        );
    }

    pub(crate) fn ensure_queue(&mut self, name: &'static str) {
        self.unique_name(name);
        self.queues.insert(name);
    }

    pub(crate) fn ensure_new(&mut self, name: &'static str) {
        self.unique_name(name);
        self.amalgam.insert(name, Default::default());
    }

    pub(crate) fn prepare_construction(&mut self, name: &'static str) {
        self.active.push(ListensAndEmits {
            name,
            emits: Vec::new(),
            listens: Vec::new(),
        });
    }

    pub(crate) fn register_emit(&mut self, signal: &'static str) {
        let last = self.active.last_mut().unwrap();
        assert!(
            last.emits.iter().find(|x| **x == signal).is_none(),
            "revent: not allowed to clone more than once per subscription: {:?}",
            signal
        );
        last.emits.push(signal);
    }

    pub(crate) fn register_subscribe(&mut self, signal: &'static str) {
        let last = self.active.last_mut().unwrap();
        assert!(
            last.listens.iter().find(|x| **x == signal).is_none(),
            "revent: not allowed to register more than once per subscription: {:?}",
            signal
        );
        last.listens.push(signal);
    }

    pub(crate) fn finish_construction(&mut self) {
        let last = self.active.pop().unwrap();

        for item in &last.listens {
            let emit = self.amalgam.entry(item).or_insert_with(Default::default);
            for emission in &last.emits {
                emit.insert(emission);
            }
        }

        for item in &last.listens {
            let listens = self.listens.entry(item).or_insert_with(Default::default);
            listens.insert(last.name);
        }

        for item in &last.emits {
            let emits = self.emits.entry(item).or_insert_with(Default::default);
            emits.insert(last.name);
        }

        match chkrec(&self.amalgam) {
            Ok(()) => {}
            Err(chain) => {
                panic!(
                    "revent: found a recursion during subscription: {}",
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

            emits: Default::default(),
            listens: Default::default(),

            queues: Default::default(),
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
                    chain.push(signal);
                    while &chain[0] != signal {
                        chain.remove(0);
                    }
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
            return Err(chain);
        }
        chain.pop();
    }
    Ok(())
}

// ---

struct RecursionPrinter<'a> {
    chain: Vec<ChannelName>,
    manager: &'a Manager,
}

impl<'a> Display for RecursionPrinter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.chain.len() < 2 {
            panic!("revent: internal error: recursion chain has length < 2");
        } else if self.chain.len() >= 2 {
            for window in self.chain.windows(2) {
                let from = window[0];
                let to = window[1];

                let emits = self.manager.emits.get(to).unwrap();
                let mut intersection = self
                    .manager
                    .listens
                    .get(from)
                    .expect(
                        "revent: internal error: recursion chain contains malformed information",
                    )
                    .intersection(emits);

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

// ---

/// Wrapper around a [Manager] that generates a graph.
pub struct Grapher<'a> {
    invemits: BTreeMap<HandlerName, BTreeSet<ChannelName>>,
    invlistens: BTreeMap<HandlerName, BTreeSet<ChannelName>>,
    queues: &'a BTreeSet<ChannelName>,
}

impl<'a> Grapher<'a> {
    /// Create a new grapher.
    pub fn new(manager: &'a Manager) -> Self {
        Self {
            invemits: Self::invert(&manager.emits),
            invlistens: Self::invert(&manager.listens),
            queues: &manager.queues,
        }
    }

    fn invert(
        map: &BTreeMap<ChannelName, BTreeSet<HandlerName>>,
    ) -> BTreeMap<HandlerName, BTreeSet<ChannelName>> {
        let mut inverse: BTreeMap<_, BTreeSet<ChannelName>> = BTreeMap::new();

        for (channel, handlers) in map {
            for handler in handlers {
                let emit = inverse.entry(*handler).or_insert_with(Default::default);
                emit.insert(channel);
            }
        }

        inverse
    }

    fn find_available_anchor_id(&self) -> String {
        let mut current = String::from("Anchor#0");
        let mut count = 0;

        while self.invlistens.contains_key(&current[..]) || self.invemits.contains_key(&current[..])
        {
            count += 1;
            current = String::from("Anchor#") + &count.to_string();
        }
        current
    }
}

impl<'a> Display for Grapher<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "strict digraph {{")?;

        let anchor_id = self.find_available_anchor_id();
        let mut colors = [
            "#3D9970", "#85144B", "#0074D9", "#2ECC40", "#FF4136", "#111111",
        ]
        .iter()
        .cycle();

        for (to, listen_channels) in &self.invlistens {
            let mut leftover = listen_channels.clone();
            for (from, emit_channels) in &self.invemits {
                let merged = listen_channels
                    .intersection(emit_channels)
                    .collect::<Vec<_>>();
                leftover = leftover.difference(emit_channels).cloned().collect();
                if !merged.is_empty() {
                    let mut merged_iter = merged.iter();
                    let color = colors.next().unwrap();
                    write!(f, "\t{:?} -> {:?}[color={:?},fontcolor={:?},label=<<FONT POINT-SIZE=\"10\">{}", from, to, color, color, merged_iter.next().unwrap())?;

                    for item in merged_iter {
                        write!(f, "<BR/>{}", item)?;
                    }
                    writeln!(f, "</FONT>>];")?;
                }
            }

            // We should also highlight signals coming from the root node that are not used by anyone
            // else.
            if !leftover.is_empty() {
                let mut iter = leftover.iter();
                let color = colors.next().unwrap();
                write!(f, "\t{:?} -> {:?}[arrowhead=\"diamond\",color={:?},fontcolor={:?},label=<<FONT POINT-SIZE=\"10\">{}", anchor_id, to, color, color, iter.next().unwrap())?;
                for left in iter {
                    write!(f, "<BR/>{}", left)?;
                }
                writeln!(f, "</FONT>>];")?;
            }
        }
        write!(f, "\t{:?}[label=<Anchor", anchor_id)?;

        if !self.queues.is_empty() {
            write!(f, "<BR/><FONT POINT-SIZE=\"10\">")?;
            for queue in self.queues.iter() {
                write!(f, "{}<BR/>", queue)?;
            }
            write!(f, "</FONT>")?;
        }
        write!(f, ">];\n}}")?;

        Ok(())
    }
}

// ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_case() {
        let mut mng = Manager::default();
        mng.prepare_construction("A");
        mng.register_emit("b");
        mng.finish_construction();

        mng.prepare_construction("B");
        mng.register_subscribe("b");
        mng.register_emit("c");
        mng.finish_construction();

        mng.prepare_construction("C");
        mng.register_subscribe("b");
        mng.register_emit("c");
        mng.finish_construction();

        let grapher = Grapher::new(&mng);
        assert_eq!(
            format!("{}", grapher),
            "strict digraph {\n\t\"A\" -> \"B\"[color=\"#3D9970\",fontcolor=\"#3D9970\",label=<<FONT POINT-SIZE=\"10\">b</FONT>>];\n\t\"A\" -> \"C\"[color=\"#85144B\",fontcolor=\"#85144B\",label=<<FONT POINT-SIZE=\"10\">b</FONT>>];\n\t\"Anchor#0\"[label=<Anchor>];\n}"
        );
    }

    #[test]
    fn graphing_queues() {
        let mut mng = Manager::default();
        mng.ensure_queue("queue");

        let grapher = Grapher::new(&mng);
        assert_eq!(
            format!("{}", grapher),
            "strict digraph {\n\t\"Anchor#0\"[label=<Anchor<BR/><FONT POINT-SIZE=\"10\">queue<BR/></FONT>>];\n}"
        );
    }
}
