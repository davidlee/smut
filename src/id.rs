//! Public identifiers and the injected source that generates them.
//!
//! [`Id`] is the core's only notion of identity: opaque text, compared and
//! ordered as text, never interpreted. [`IdSource`] is the core's only way to
//! obtain a fresh candidate, so the core itself holds no entropy (DEC-001).
//!
//! The collision contract lives here too. [`Occupied`] is the set a candidate must
//! avoid — existing entity ids, every explicit id in the request, and every id
//! generated earlier in the same batch — and it is *passed in*, never discovered
//! from a source, because a generated collision that slipped through would become
//! an accidental upsert (DEC-001 rule 3).

use std::borrow::Borrow;
use std::collections::BTreeSet;
use std::fmt;

/// An entity's public id: opaque, case-sensitive text, unnormalised.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Id(String);

impl Id {
    /// Wraps a public id.
    pub fn new(value: impl Into<String>) -> Self {
        Id(value.into())
    }

    /// The id's text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for Id {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl From<&str> for Id {
    fn from(value: &str) -> Self {
        Id::new(value)
    }
}

impl From<String> for Id {
    fn from(value: String) -> Self {
        Id::new(value)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// The core's only way to obtain a fresh id candidate (DEC-001).
///
/// The source offers candidates; it does not decide whether they are usable. The
/// caller draws from it until it finds one outside the [`Occupied`] set, so a
/// source that repeats itself is still safe and a source that never yields a free
/// candidate loops by contract, not by accident.
pub trait IdSource {
    /// Offers the next candidate, in whatever order the source chooses.
    fn candidate(&mut self) -> String;
}

/// The ids a generated candidate must avoid: the snapshot's keys, every explicit
/// id in the request, and every id generated earlier in the same batch.
#[derive(Clone, Debug, Default)]
pub struct Occupied {
    ids: BTreeSet<Id>,
}

impl Occupied {
    /// No occupied ids yet.
    pub fn new() -> Self {
        Occupied::default()
    }

    /// The ids already occupied by an in-memory snapshot.
    pub fn of<'a>(existing: impl IntoIterator<Item = &'a Id>) -> Self {
        Occupied {
            ids: existing.into_iter().cloned().collect(),
        }
    }

    /// Records one more id as occupied.
    pub fn occupy(&mut self, id: Id) {
        self.ids.insert(id);
    }

    /// Whether `candidate` is already occupied.
    pub fn holds(&self, candidate: &str) -> bool {
        self.ids.contains(candidate)
    }

    /// Draws candidates until one is unoccupied, records it, and returns it.
    pub fn generate(&mut self, source: &mut dyn IdSource) -> Id {
        loop {
            let candidate = source.candidate();

            if !self.holds(&candidate) {
                let id = Id::new(candidate);
                self.occupy(id.clone());

                return id;
            }
        }
    }
}

/// A deterministic [`IdSource`] for tests: it yields its script in order and
/// reports how much of the script a caller consumed.
#[cfg(test)]
pub(crate) mod scripted {
    use super::IdSource;

    /// A source that hands back pre-chosen candidates.
    #[derive(Debug)]
    pub struct ScriptedSource {
        candidates: Vec<String>,
        drawn: usize,
    }

    impl ScriptedSource {
        /// A source yielding `candidates` in the given order.
        pub fn new(candidates: &[&str]) -> Self {
            ScriptedSource {
                candidates: candidates.iter().map(|text| (*text).to_owned()).collect(),
                drawn: 0,
            }
        }

        /// How many candidates have been handed out.
        pub fn draws(&self) -> usize {
            self.drawn
        }
    }

    impl IdSource for ScriptedSource {
        fn candidate(&mut self) -> String {
            let candidate = self
                .candidates
                .get(self.drawn)
                .unwrap_or_else(|| panic!("the scripted id source is exhausted"))
                .clone();
            self.drawn += 1;

            candidate
        }
    }
}

#[cfg(test)]
mod tests {
    use super::scripted::ScriptedSource;
    use super::*;

    #[test]
    fn an_id_is_its_text_for_equality_and_ordering() {
        assert_eq!(Id::new("a"), Id::new("a".to_owned()));
        assert_ne!(Id::new("a"), Id::new("A"));
        assert_eq!(Id::new("smut").as_str(), "smut");
        assert_eq!(Id::new("smut").to_string(), "smut");
        assert!(Id::new("a") < Id::new("b"));
    }

    #[test]
    fn generation_avoids_the_snapshots_occupied_ids() {
        // The forced collision: the first candidate is an id the snapshot holds.
        let snapshot = [Id::new("stored")];
        let mut occupied = Occupied::of(&snapshot);
        let mut source = ScriptedSource::new(&["stored", "fresh"]);

        assert_eq!(occupied.generate(&mut source), Id::new("fresh"));
        assert_eq!(source.draws(), 2);
    }

    #[test]
    fn generation_avoids_an_explicit_id_in_the_request() {
        let mut occupied = Occupied::new();
        occupied.occupy(Id::new("explicit"));
        let mut source = ScriptedSource::new(&["explicit", "fresh"]);

        assert_eq!(occupied.generate(&mut source), Id::new("fresh"));
        assert_eq!(source.draws(), 2);
    }

    #[test]
    fn generation_avoids_an_earlier_generated_id() {
        let mut occupied = Occupied::new();
        let mut source = ScriptedSource::new(&["reused", "reused", "fresh"]);

        assert_eq!(occupied.generate(&mut source), Id::new("reused"));
        assert_eq!(occupied.generate(&mut source), Id::new("fresh"));
        assert_eq!(source.draws(), 3);
    }

    #[test]
    fn an_empty_occupied_set_accepts_the_first_candidate() {
        let mut occupied = Occupied::new();
        let mut source = ScriptedSource::new(&["first"]);

        assert_eq!(occupied.generate(&mut source), Id::new("first"));
    }

    #[test]
    fn the_occupied_set_reports_what_it_holds() {
        let mut occupied = Occupied::new();
        assert!(!occupied.holds("a"));

        occupied.occupy(Id::new("a"));

        assert!(occupied.holds("a"));
        assert!(!occupied.holds("b"));
    }
}
