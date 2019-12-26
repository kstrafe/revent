use std::{
    collections::{BTreeMap, HashSet},
    error::Error,
    fmt::{self, Debug, Display},
};

#[derive(PartialEq)]
pub struct RecursionError {
    chain: Vec<&'static str>,
}

impl Debug for RecursionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RecursionError {{ chain: {:?} }}", self.chain)
    }
}

impl Display for RecursionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Recursion found {:?}", self.chain)
    }
}

impl Error for RecursionError {}

#[derive(PartialEq)]
pub struct ChainedError {
    from: &'static str,
    to: &'static str,
}

impl Debug for ChainedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ChainedError {{ from: {}, to: {} }}", self.from, self.to)
    }
}

impl Display for ChainedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Subscriber chain found: {} can call {}",
            self.from, self.to
        )
    }
}

impl Error for ChainedError {}

/// Computes signal recursions.
#[derive(Default)]
pub struct Recursion {
    mapping: BTreeMap<&'static str, Vec<&'static str>>,
}

impl Recursion {
    /// Add a parent and its children signals.
    pub fn add(&mut self, parent: &'static str, child: &[&'static str]) {
        let children = self.mapping.entry(parent).or_insert_with(Vec::new);
        children.extend(child);
    }

    /// Check if there is any recursion present.
    pub fn check(&mut self) -> Result<(), RecursionError> {
        let mut chain = Vec::new();
        for parent in self.mapping.keys() {
            chain.push(*parent);
            self.check_internal(parent, &mut chain)?;
            chain.pop();
        }
        Ok(())
    }

    fn check_internal(
        &self,
        parent: &'static str,
        chain: &mut Vec<&'static str>,
    ) -> Result<(), RecursionError> {
        if let Some(children) = self.mapping.get(parent) {
            for child in children {
                if let Some((idx, _)) = chain.iter().enumerate().find(|(_, x)| x == &child) {
                    return Err(RecursionError {
                        chain: chain[idx..].to_vec(),
                    });
                }
                chain.push(child);
                self.check_internal(child, chain)?;
                chain.pop();
            }
        }
        Ok(())
    }

    /// Check if subscribing to a set of signals can cause an N-mutable borrow for this subscriber.
    pub fn is_chained(&self, signals: &[&'static str]) -> Result<(), ChainedError> {
        for (idx, signal) in signals.iter().enumerate() {
            let mut set = HashSet::new();
            self.collect_descendants(signal, &mut set);
            for (redex, to_signal) in signals.iter().enumerate() {
                if redex == idx {
                    continue;
                }
                if set.contains(to_signal) {
                    return Err(ChainedError {
                        from: signal,
                        to: to_signal,
                    });
                }
            }
        }
        Ok(())
    }

    fn collect_descendants(&self, parent: &'static str, set: &mut HashSet<&'static str>) {
        if let Some(children) = self.mapping.get(parent) {
            for child in children {
                if !set.contains(child) {
                    self.collect_descendants(child, set);
                }
                set.insert(child);
            }
        } else {
            panic!("Node {:?} is not registered", parent);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_recursion() {
        let mut rec = Recursion::default();
        rec.add("A", &["B", "C"]);
        rec.add("B", &["C"]);
        rec.add("D", &["C"]);
        assert!(rec.check().is_ok());
    }

    #[test]
    fn self_recursion() {
        let mut rec = Recursion::default();
        rec.add("A", &["A"]);
        assert_eq!(Err(RecursionError { chain: vec!["A"] }), rec.check());
    }

    #[test]
    fn simplest_transitive_recursion() {
        let mut rec = Recursion::default();
        rec.add("A", &["B"]);
        rec.add("B", &["A"]);
        assert_eq!(
            Err(RecursionError {
                chain: vec!["A", "B"]
            }),
            rec.check()
        );
    }

    #[test]
    fn long_non_recursive_chain() {
        let mut rec = Recursion::default();
        rec.add("A", &["B"]);
        rec.add("B", &["C"]);
        rec.add("C", &["D"]);
        rec.add("D", &["E"]);
        rec.add("E", &["F"]);
        assert!(rec.check().is_ok());
    }

    #[test]
    fn long_recursive_chain() {
        let mut rec = Recursion::default();
        rec.add("A", &["B"]);
        rec.add("B", &["C"]);
        rec.add("C", &["D"]);
        rec.add("D", &["E"]);
        rec.add("E", &["A"]);
        assert_eq!(
            Err(RecursionError {
                chain: vec!["A", "B", "C", "D", "E"]
            }),
            rec.check()
        );
    }

    #[test]
    fn chained_subscriber() {
        let mut rec = Recursion::default();
        rec.add("A", &["B"]);
        rec.add("B", &["C"]);
        rec.add("C", &[]);
        rec.check().unwrap();
        assert_eq!(
            Err(ChainedError { from: "A", to: "C" }),
            rec.is_chained(&["A", "C"])
        );
        assert_eq!(Ok(()), rec.is_chained(&["A"]));
        assert_eq!(Ok(()), rec.is_chained(&["B"]));
        assert_eq!(Ok(()), rec.is_chained(&["C"]));
    }
}
