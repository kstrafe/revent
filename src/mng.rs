#[cfg(feature = "logging")]
use slog::{o, trace, Discard, Logger};
#[cfg(feature = "logging")]
use std::collections::HashMap;
use std::{
    cell::{Ref, RefCell},
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Debug, Display, Formatter},
    fs, io,
    path::Path,
    process::Command,
    rc::Rc,
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

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ChannelType {
    Direct,
    Feed,
}

impl ChannelType {
    fn is_direct(&self) -> bool {
        match self {
            ChannelType::Direct => true,
            _ => false,
        }
    }
}

/// Inspects how various [Node](crate::Node)s use slots.
///
/// Manager is active when subscribing to [Anchor](crate::Anchor). It inspects the various listens and emits and
/// ensures that there are no cycles created between nodes.
///
/// Unsubscribing items does not remove the channel dependencies from the manager. This is
/// intentional to discourage juggling subscriptions to fit the dependency chain.
///
/// # Panics #
///
/// Will [panic] if there exists any subscriber cycle. Cycle detection occurs only during
/// [Anchor::subscribe](crate::Anchor::subscribe). Emitting will not perform any cycle detection.
#[derive(Clone, Debug)]
pub struct Manager(pub(crate) Rc<RefCell<ManagerInternal>>);

#[derive(Debug)]
pub(crate) struct ManagerInternal {
    active: Vec<ListensAndEmits>,
    amalgam: BTreeMap<ChannelName, BTreeSet<ChannelName>>,

    emits: BTreeMap<ChannelName, BTreeSet<HandlerName>>,
    listens: BTreeMap<ChannelName, BTreeSet<HandlerName>>,

    channel_types: BTreeMap<ChannelName, ChannelType>,

    #[cfg(feature = "logging")]
    names: HashMap<*const (), HandlerName>,
    #[cfg(feature = "logging")]
    logger: Logger,
    #[cfg(feature = "logging")]
    emit_level: usize,
}

impl ManagerInternal {
    fn unique_name(&self, name: &'static str) {
        assert!(
            !self.channel_types.contains_key(name),
            "revent: name is already registered to this manager: {:?}",
            name
        );
    }

    fn chkrec(&self) -> Result<(), Vec<ChannelName>> {
        let set = &self.amalgam;
        fn chkreci(
            now: ChannelName,
            set: &BTreeMap<ChannelName, BTreeSet<ChannelName>>,
            chain: &mut Vec<ChannelName>,
            channel_types: &BTreeMap<ChannelName, ChannelType>,
        ) -> Result<(), ()> {
            if let Some(node) = set.get(&now) {
                for signal in node
                    .iter()
                    .filter(|x| channel_types.get(*x).unwrap().is_direct())
                {
                    if chain.contains(&signal) {
                        chain.push(signal);
                        while &chain[0] != signal {
                            chain.remove(0);
                        }
                        return Err(());
                    }
                    chain.push(*signal);
                    chkreci(signal, set, chain, channel_types)?;
                    chain.pop();
                }
            }
            Ok(())
        }

        let mut chain = Vec::new();
        for signal in set
            .keys()
            .filter(|x| self.channel_types.get(*x).unwrap().is_direct())
        {
            chain.push(*signal);
            if let Err(()) = chkreci(signal, set, &mut chain, &self.channel_types) {
                return Err(chain);
            }
            chain.pop();
        }
        Ok(())
    }
}

impl Manager {
    /// Create a new manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new manager with a logger.
    #[cfg(feature = "logging")]
    pub fn with_logger(logger: Logger) -> Self {
        Self(Rc::new(RefCell::new(ManagerInternal {
            active: Default::default(),
            amalgam: Default::default(),

            emits: Default::default(),
            listens: Default::default(),

            channel_types: Default::default(),

            names: Default::default(),
            logger,
            emit_level: 0,
        })))
    }

    #[cfg(feature = "logging")]
    pub(crate) fn log_emit(&self, name: ChannelName) {
        let mut this = self.0.borrow_mut();
        if this.emit_level == 0 {
            trace!(this.logger, "Root emit"; "channel" => name);
        }
        this.emit_level += 1;
    }

    #[cfg(feature = "logging")]
    pub(crate) fn log_feedee(&self, name: ChannelName) {
        let this = self.0.borrow_mut();
        trace!(
            this.logger,
            "{}<- {}",
            "\t".repeat(this.emit_level + 1),
            name
        );
    }

    #[cfg(feature = "logging")]
    pub(crate) fn log_feeder(&self, name: ChannelName) {
        let this = self.0.borrow_mut();
        trace!(
            this.logger,
            "{}-> {}",
            "\t".repeat(this.emit_level + 1),
            name
        );
    }

    #[cfg(feature = "logging")]
    pub(crate) fn log_feeder_push(&self, recipient: HandlerName) {
        let this = self.0.borrow_mut();
        trace!(
            this.logger,
            "{}{}",
            "\t".repeat(this.emit_level + 2),
            recipient
        );
    }

    #[cfg(feature = "logging")]
    pub(crate) fn log_emit_on_item<T: ?Sized>(&self, item: Rc<RefCell<T>>, channel: ChannelName) {
        let this = self.0.borrow();
        let ptr = Rc::into_raw(item) as *const ();
        unsafe {
            Rc::from_raw(ptr);
        }
        let handler_name = this.names.get(&ptr).unwrap();
        trace!(
            this.logger,
            "{}{} ({})",
            "\t".repeat(this.emit_level),
            handler_name,
            channel
        );
    }

    #[cfg(feature = "logging")]
    pub(crate) fn log_register<T: ?Sized>(&self, name: HandlerName, item: Rc<RefCell<T>>) {
        let mut this = self.0.borrow_mut();
        let ptr = Rc::into_raw(item) as *const ();
        unsafe {
            Rc::from_raw(ptr);
        }
        assert!(matches!(this.names.insert(ptr, name), None));
    }

    #[cfg(feature = "logging")]
    pub(crate) fn log_deregister<T: ?Sized>(&self, name: HandlerName, item: Rc<RefCell<T>>) {
        let mut this = self.0.borrow_mut();
        let ptr = Rc::into_raw(item) as *const ();
        unsafe {
            Rc::from_raw(ptr);
        }
        assert_eq!(this.names.remove(&ptr).unwrap(), name);
    }

    #[cfg(feature = "logging")]
    pub(crate) fn log_emit_end(&self) {
        let mut this = self.0.borrow_mut();
        this.emit_level -= 1;
    }

    pub(crate) fn current(&self) -> HandlerName {
        let this = &mut *self.0.borrow_mut();
        this.active.last().unwrap().name
    }

    pub(crate) fn ensure_new(&self, name: &'static str, channel_type: ChannelType) {
        let this = &mut *self.0.borrow_mut();

        this.unique_name(name);
        this.channel_types.insert(name, channel_type);
    }

    pub(crate) fn prepare_construction(&self, name: &'static str) {
        let this = &mut *self.0.borrow_mut();

        this.active.push(ListensAndEmits {
            name,
            emits: Vec::new(),
            listens: Vec::new(),
        });
    }

    pub(crate) fn register_emit(&self, signal: &'static str) {
        let this = &mut *self.0.borrow_mut();

        let last = this.active.last_mut().unwrap();
        assert!(
            last.emits.iter().find(|x| **x == signal).is_none(),
            "revent: not allowed to clone more than once per subscription: {:?}",
            signal
        );
        last.emits.push(signal);
    }

    pub(crate) fn register_listen(&self, signal: &'static str) {
        let this = &mut *self.0.borrow_mut();

        let last = this.active.last_mut().unwrap();
        assert!(
            last.listens.iter().find(|x| **x == signal).is_none(),
            "revent: not allowed to register more than once per subscription: {:?}",
            signal
        );
        last.listens.push(signal);
    }

    pub(crate) fn finish_construction(&self) {
        let this = &mut *self.0.borrow_mut();

        let last = this.active.pop().unwrap();

        for item in &last.listens {
            let emit = this.amalgam.entry(item).or_insert_with(Default::default);
            for emission in &last.emits {
                emit.insert(emission);
            }
        }

        for item in &last.listens {
            let listens = this.listens.entry(item).or_insert_with(Default::default);
            listens.insert(last.name);
        }

        for item in &last.emits {
            let emits = this.emits.entry(item).or_insert_with(Default::default);
            emits.insert(last.name);
        }

        match this.chkrec() {
            Ok(()) => {}
            Err(chain) => {
                panic!(
                    "revent: found a recursion during subscription: {}",
                    RecursionPrinter {
                        chain,
                        manager: &*this,
                    }
                );
            }
        }
    }
}

impl Default for Manager {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(ManagerInternal {
            active: Default::default(),
            amalgam: Default::default(),

            emits: Default::default(),
            listens: Default::default(),

            channel_types: Default::default(),

            #[cfg(feature = "logging")]
            names: Default::default(),
            #[cfg(feature = "logging")]
            logger: Logger::root(Discard, o!()),
            #[cfg(feature = "logging")]
            emit_level: 0,
        })))
    }
}

// ---

struct RecursionPrinter<'a> {
    chain: Vec<ChannelName>,
    manager: &'a ManagerInternal,
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
    internal: Ref<'a, ManagerInternal>,
}

impl<'a> Grapher<'a> {
    /// Create a new grapher.
    pub fn new(manager: &'a Manager) -> Self {
        Self {
            invemits: Self::invert(&manager.0.borrow().emits),
            invlistens: Self::invert(&manager.0.borrow().listens),
            internal: manager.0.borrow(),
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

    fn find_available_anchor_id(&self, mut count_start: usize) -> (String, usize) {
        let mut current = format!("Anchor#{}", count_start);

        while self.invlistens.contains_key(&current[..]) || self.invemits.contains_key(&current[..])
        {
            count_start += 1;
            current = format!("Anchor#{}", count_start);
        }
        (current, count_start)
    }

    /// Run `dot` on the graph to generate a `png` file.
    pub fn graph_to_file<P: AsRef<Path>>(&self, filename: P) -> Result<(), io::Error> {
        let filename = filename.as_ref();
        let dot_file = filename.with_extension("dot");
        fs::write(&dot_file, format!("{}", self))?;
        fs::write(
            filename,
            Command::new("dot")
                .args(&[dot_file.to_str().unwrap(), "-T", "png"])
                .output()?
                .stdout,
        )
    }
}

impl<'a> Display for Grapher<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "strict digraph {{")?;

        let mut anchor_count = 0;
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
                    // Create direct links first
                    let mut merged_iter = merged
                        .iter()
                        .filter(|&x| self.internal.channel_types.get(*x).unwrap().is_direct());

                    if let Some(item) = merged_iter.next() {
                        let color = colors.next().unwrap();
                        write!(f, "\t{:?} -> {:?}[color={:?},fontcolor={:?},label=<<FONT POINT-SIZE=\"10\">{}", from, to, color, color, item)?;

                        for item in merged_iter {
                            write!(f, "<BR/>{}", item)?;
                        }
                        writeln!(f, "</FONT>>];")?;
                    }

                    let mut merged_iter = merged
                        .iter()
                        .filter(|&x| !self.internal.channel_types.get(*x).unwrap().is_direct());
                    if let Some(item) = merged_iter.next() {
                        let color = colors.next().unwrap();
                        write!(f, "\t{:?} -> {:?}[color={:?},fontcolor={:?},label=<<FONT POINT-SIZE=\"10\">{}", from, to, color, color, item)?;

                        for item in merged_iter {
                            write!(f, "<BR/>{}", item)?;
                        }
                        writeln!(f, "</FONT>>,style=\"dashed\"];")?;
                    }
                }
            }

            // We should also highlight signals coming from the root node that are not used by anyone
            // else.
            if !leftover.is_empty() {
                let mut iter = leftover.iter();
                let color = colors.next().unwrap();
                let (anchor_name, new_count) = self.find_available_anchor_id(anchor_count);
                anchor_count = new_count + 1;
                anchor_count += 1;
                write!(f, "\t{:?}[style=\"invis\"];", anchor_name)?;
                write!(f, "\t{:?} -> {:?}[arrowhead=\"diamond\",color={:?},fontcolor={:?},label=<<FONT POINT-SIZE=\"10\">{}", anchor_name, to, color, color, iter.next().unwrap())?;
                for left in iter {
                    write!(f, "<BR/>{}", left)?;
                }
                writeln!(f, "</FONT>>];")?;
            }
        }

        write!(f, "\n}}")?;

        Ok(())
    }
}

// ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_case() {
        let mng = Manager::new();
        mng.ensure_new("b", ChannelType::Direct);
        mng.ensure_new("c", ChannelType::Direct);

        mng.prepare_construction("A");
        mng.register_emit("b");
        mng.finish_construction();

        mng.prepare_construction("B");
        mng.register_listen("b");
        mng.register_emit("c");
        mng.finish_construction();

        mng.prepare_construction("C");
        mng.register_listen("b");
        mng.register_emit("c");
        mng.finish_construction();

        let grapher = Grapher::new(&mng);
        assert_eq!(
            format!("{}", grapher),
            "strict digraph {\n\t\"A\" -> \"B\"[color=\"#3D9970\",fontcolor=\"#3D9970\",label=<<FONT POINT-SIZE=\"10\">b</FONT>>];\n\t\"A\" -> \"C\"[color=\"#85144B\",fontcolor=\"#85144B\",label=<<FONT POINT-SIZE=\"10\">b</FONT>>];\n\n}"
        );
    }

    #[test]
    fn graph_feed_loop() {
        let mng = Manager::new();
        mng.ensure_new("a", ChannelType::Direct);
        mng.ensure_new("b", ChannelType::Feed);

        mng.prepare_construction("A");
        mng.register_emit("a");
        mng.register_listen("b");
        mng.finish_construction();

        mng.prepare_construction("B");
        mng.register_listen("a");
        mng.register_emit("b");
        mng.finish_construction();

        let grapher = Grapher::new(&mng);
        assert_eq!(
            format!("{}", grapher),
            "strict digraph {\n\t\"B\" -> \"A\"[color=\"#3D9970\",fontcolor=\"#3D9970\",label=<<FONT POINT-SIZE=\"10\">b</FONT>>,style=\"dashed\"];\n\t\"A\" -> \"B\"[color=\"#85144B\",fontcolor=\"#85144B\",label=<<FONT POINT-SIZE=\"10\">a</FONT>>];\n\n}"
        );
    }

    #[test]
    fn include_anchor_if_signals_unaccounted() {
        let mng = Manager::new();
        mng.ensure_new("a", ChannelType::Direct);

        mng.prepare_construction("A");
        mng.register_listen("a");
        mng.finish_construction();

        let grapher = Grapher::new(&mng);
        assert_eq!(
            format!("{}", grapher),
            "strict digraph {\n\t\"Anchor#0\"[style=\"invis\"];\t\"Anchor#0\" -> \"A\"[arrowhead=\"diamond\",color=\"#3D9970\",fontcolor=\"#3D9970\",label=<<FONT POINT-SIZE=\"10\">a</FONT>>];\n\n}"
        );
    }
}
