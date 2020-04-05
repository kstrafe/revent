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

/// Inspects how various [Subscriber](crate::Subscriber)s use [Slot](crate::Slot)s.
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

    emitters: BTreeMap<ChannelName, BTreeSet<HandlerName>>,
    listens: BTreeMap<ChannelName, BTreeSet<HandlerName>>,
}

impl Manager {
    /// Create a new manager.
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn ensure_new(&mut self, name: &'static str) {
        assert!(
            !self.emitters.contains_key(name),
            "revent: name is already registered to this manager: {:?}",
            name
        );
        self.emitters.insert(name, Default::default());
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
            let emits = self.emitters.entry(item).or_insert_with(Default::default);
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

            emitters: Default::default(),
            listens: Default::default(),
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

                dbg!(to);
                let emitters = self.manager.emitters.get(to).unwrap();
                let mut intersection = self
                    .manager
                    .listens
                    .get(from)
                    .expect(
                        "revent: internal error: recursion chain contains malformed information",
                    )
                    .intersection(emitters);

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
    manager: &'a Manager,
}

impl<'a> Grapher<'a> {
    /// Create a new grapher.
    pub fn new(manager: &'a Manager) -> Self {
        Self { manager }
    }
}

impl<'a> Display for Grapher<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mng = self.manager;

        writeln!(f, "digraph Manager {{")?;

        for (channel, handlers) in &mng.listens {
            write!(
                f,
                "\t{}[label=<<FONT POINT-SIZE=\"20\">{}</FONT>",
                channel, channel
            )?;
            for handler in handlers {
                write!(f, "<BR/>{}", handler)?;
            }
            writeln!(f, ">];")?;
        }

        for (from, to) in &mng.amalgam {
            for to in to {
                writeln!(f, "\t{} -> {};", from, to)?;
            }
        }

        write!(f, "}}")?;

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
            r#"digraph Manager {
	b[label=<<FONT POINT-SIZE="20">b</FONT><BR/>B<BR/>C>];
	b -> c;
}"#
        );
    }
}
