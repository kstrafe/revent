#![doc(hidden)]
use std::{
    any::TypeId,
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Display, Formatter},
};

type ChannelName = &'static str;
type HandlerName = &'static str;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct HandlerId {
    name: HandlerName,
    type_id: TypeId,
}

enum ActiveHandler {
    Id { id: HandlerId },
    Ignore,
}

pub struct Manager {
    active: Vec<ActiveHandler>,
    amalgam: BTreeMap<ChannelName, BTreeSet<ChannelName>>,
    connections: BTreeMap<HandlerId, (BTreeSet<ChannelName>, BTreeSet<ChannelName>)>,

    emitters: BTreeMap<ChannelName, BTreeSet<HandlerId>>,
    subscribers: BTreeMap<ChannelName, BTreeSet<HandlerId>>,

    seen: BTreeSet<TypeId>,
}

impl Manager {
    pub fn prepare_construction(&mut self, name: &'static str, type_id: TypeId) {
        if self.seen.contains(&type_id) {
            self.active.push(ActiveHandler::Ignore);
        } else {
            self.seen.insert(type_id);
            self.active.push(ActiveHandler::Id {
                id: HandlerId { name, type_id },
            });
        }
    }

    pub fn register_emit(&mut self, signal: &'static str) {
        let name = self.active.last().unwrap();
        match name {
            ActiveHandler::Id { id } => {
                let connection = self.connections.entry(*id).or_insert_with(Default::default);
                connection.1.insert(signal);

                self.emitters
                    .entry(signal)
                    .or_insert_with(Default::default)
                    .insert(*id);
            }
            ActiveHandler::Ignore => {}
        }
    }

    pub fn register_subscribe(&mut self, signal: &'static str) {
        let name = self.active.last().unwrap();
        match name {
            ActiveHandler::Id { id } => {
                let connection = self.connections.entry(*id).or_insert_with(Default::default);
                connection.0.insert(signal);

                self.subscribers
                    .entry(signal)
                    .or_insert_with(Default::default)
                    .insert(*id);
            }
            ActiveHandler::Ignore => {}
        }
    }

    pub fn finish_construction(&mut self) {
        let name = self.active.pop().unwrap();
        match name {
            ActiveHandler::Id { id } => {
                let connection = self.connections.entry(id).or_insert_with(Default::default);
                for item in &connection.0 {
                    let emit = self.amalgam.entry(item).or_insert_with(Default::default);
                    for emission in &connection.1 {
                        emit.insert(emission);
                    }
                }

                match chkrec(&self.amalgam) {
                    Ok(()) => {}
                    Err(chain) => {
                        panic!(
                            "revent found a recursion during subscription: {}",
                            RecursionPrinter {
                                chain,
                                manager: self,
                            }
                        );
                    }
                }
            }
            ActiveHandler::Ignore => {}
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

            seen: Default::default(),
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

struct RecursionPrinter<'a> {
    chain: Vec<ChannelName>,
    manager: &'a Manager,
}

impl<'a> Display for RecursionPrinter<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.chain.len() < 2 {
            panic!("revent internal error: recursion chain has length < 2");
        } else if self.chain.len() >= 2 {
            let mut name_enumerator = HandlerEnumerator::default();

            for window in self.chain.windows(2) {
                let from = window[0];
                let to = window[1];

                let emitters = self.manager.emitters.get(to).unwrap();
                let mut intersection = self
                    .manager
                    .subscribers
                    .get(from)
                    .expect("revent internal error: recursion chain contains malformed information")
                    .intersection(emitters);

                write!(f, "[")?;
                if let Some(item) = intersection.next() {
                    write!(f, "{}{}", item.name, name_enumerator.enumerate_name(*item))?;
                }
                for item in intersection {
                    write!(
                        f,
                        ", {}{}",
                        item.name,
                        name_enumerator.enumerate_name(*item)
                    )?;
                }
                write!(f, "]{} -> ", from)?;
            }

            write!(f, "{}", self.chain.last().unwrap())?;
        }
        Ok(())
    }
}

#[derive(Default)]
struct HandlerEnumerator {
    type_count: BTreeMap<HandlerName, BTreeMap<TypeId, usize>>,
}

impl HandlerEnumerator {
    fn enumerate_name(&mut self, id: HandlerId) -> MaybeUsize {
        if let Some(value) = self
            .type_count
            .entry(id.name)
            .or_insert_with(Default::default)
            .get(&id.type_id)
        {
            (*value).into()
        } else {
            let count = self.type_count.get(id.name).unwrap().len();
            self.type_count
                .get_mut(id.name)
                .unwrap()
                .insert(id.type_id, count);
            count.into()
        }
    }
}

enum MaybeUsize {
    Value(usize),
    Nothing,
}

impl From<usize> for MaybeUsize {
    fn from(item: usize) -> Self {
        if item == 0 {
            MaybeUsize::Nothing
        } else {
            MaybeUsize::Value(item)
        }
    }
}

impl Display for MaybeUsize {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Value(value) => write!(f, "#{}", value)?,
            Self::Nothing => {}
        }
        Ok(())
    }
}

pub struct Grapher<'a> {
    manager: &'a Manager,
}

impl<'a> Grapher<'a> {
    pub fn new(manager: &'a Manager) -> Self {
        Self { manager }
    }
}

impl<'a> Display for Grapher<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mng = self.manager;

        writeln!(f, "digraph Manager {{")?;

        for (channel, subscribers) in &mng.subscribers {
            write!(
                f,
                "\t{}[label=<<FONT POINT-SIZE=\"20\">{}</FONT>",
                channel, channel
            )?;
            for subscriber in subscribers {
                write!(f, "<BR/>{}", subscriber.name)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    struct A;
    struct B;
    struct C;

    #[test]
    fn make_graph() {
        let mut mng = Manager::default();
        mng.prepare_construction("A", TypeId::of::<A>());
        mng.register_emit("b");
        mng.finish_construction();

        mng.prepare_construction("B", TypeId::of::<B>());
        mng.register_subscribe("b");
        mng.register_emit("c");
        mng.finish_construction();

        mng.prepare_construction("C", TypeId::of::<C>());
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
