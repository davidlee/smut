//! The traversal: a breadth-first walk over full entities.
//!
//! The walk runs on the snapshot's **full entities**, after the roots are
//! resolved and before projection. That ordering is load-bearing twice: a
//! `select` that hides a reference field must not erase connectivity, and the
//! result limit must not widen what the walk discovered (§4, §7.1).
//!
//! Everything a walk reports borrows from the snapshot: ids, field labels and
//! dangling targets are `&'a` references into it, and only the collections that
//! order them are owned. [`Walk`] deliberately holds no `&Entities`: its ordered
//! id list is the whole answer, and whoever receives a walk cannot widen it by
//! re-reading the store (DEC-007).
//!
//! References are read one way only. Every pair this module considers comes from
//! [`crate::entity::scan_refs`], the crate's single definition of what a
//! reference is; nothing here inspects JSON to find ids, so a nested or embedded
//! reference is discovered exactly as the envelope's edge derivation later sees
//! it (DEC-005).
//!
//! The walk does not record an edge set. Edges are a property of the *returned*
//! objects — the induced subgraph, derived in the envelope from their full
//! entities after the result limit — not of the expansions that happened to
//! discover them, so a returned node that was never expanded still contributes
//! its edges (DEC-014).
//!
//! Two costs are whole-store and accepted at this store size (§10.2): the root
//! scan outside this module, and the store scan every incoming expansion
//! performs. No index is invented here.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use serde_json::Value;

use crate::declare::Entities;
use crate::entity::scan_refs;
use crate::id::Id;

/// Which outgoing or incoming fields a traversal asks for (§5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fields {
    /// `"*"`: every top-level field, in [`scan_refs`] order.
    All,
    /// An array of top-level field names, in query order; duplicates behave as
    /// a set but never reorder the first occurrence.
    Named(Vec<String>),
}

/// One requested-field reference to an entity that is not stored (§7.2).
///
/// The triple is the identity: two encounters with the same `(source, field,
/// target)` are one dangling reference, and two absent targets of the same
/// source are two.
#[derive(Clone, Debug, PartialEq)]
pub struct Dangling<'a> {
    /// The expanded entity that carries the reference.
    pub source: &'a Id,
    /// The reference's top-level field name, never a nested key.
    pub field: &'a str,
    /// The id the reference names, which the store does not hold.
    pub target: &'a str,
}

/// The walk's own account of what it did (§4, §7).
#[derive(Clone, Debug, PartialEq)]
pub struct Walk<'a> {
    /// Discovery order, deduped; roots first, in root order.
    pub order: Vec<&'a Id>,
    /// Every discovered id and its minimum discovery depth; roots are 0.
    pub depths: BTreeMap<&'a Id, u64>,
    /// Discovered but never expanded, in discovery order (§7.2, DEC-013).
    pub frontier: Vec<&'a Id>,
    /// Requested-field references to absent entities, deduped, in encounter
    /// order.
    pub dangling: Vec<Dangling<'a>>,
    /// A permitted expansion was declined by the budget.
    pub exhausted: bool,
}

/// Walks the snapshot breadth-first from `roots`, following `follow` outgoing and
/// `backlinks` incoming requested fields out to `depth` hops, expanding at most
/// `budget` entities.
///
/// Every root must be stored: callers resolve roots against the snapshot first,
/// so a missing one is a precondition breach and panics rather than corrupting
/// [`Walk::order`].
pub fn walk<'a>(
    store: &'a Entities,
    roots: &[&'a Id],
    follow: Option<&Fields>,
    backlinks: Option<&Fields>,
    depth: u64,
    budget: u64,
) -> Walk<'a> {
    let mut walker = Walker::new(store, follow, backlinks, depth, budget);
    walker.run(roots);
    walker.finish()
}

/// The walk in progress: the accumulators [`Walk`] reports, plus the queue and
/// the dedupe guards that exist only while the walk runs.
struct Walker<'a, 'f> {
    store: &'a Entities,
    follow: Option<&'f Fields>,
    backlinks: Option<&'f Fields>,
    depth: u64,
    budget: u64,
    order: Vec<&'a Id>,
    depths: BTreeMap<&'a Id, u64>,
    frontier: Vec<&'a Id>,
    dangling: Vec<Dangling<'a>>,
    exhausted: bool,
    /// Every discovered id: the guard that makes an entity expand at most once.
    seen: BTreeSet<&'a Id>,
    /// Discovery order, with the depth each id was discovered at.
    queue: VecDeque<(&'a Id, u64)>,
    /// The dangling triples already recorded, keeping [`Self::dangling`]'s
    /// encounter order deduplicated.
    dangling_seen: BTreeSet<(&'a Id, &'a str, &'a str)>,
}

impl<'a, 'f> Walker<'a, 'f> {
    fn new(
        store: &'a Entities,
        follow: Option<&'f Fields>,
        backlinks: Option<&'f Fields>,
        depth: u64,
        budget: u64,
    ) -> Self {
        Walker {
            store,
            follow,
            backlinks,
            depth,
            budget,
            order: Vec::new(),
            depths: BTreeMap::new(),
            frontier: Vec::new(),
            dangling: Vec::new(),
            exhausted: false,
            seen: BTreeSet::new(),
            queue: VecDeque::new(),
            dangling_seen: BTreeSet::new(),
        }
    }

    /// §7.1's loop: roots in root order, then breadth-first over the queue.
    fn run(&mut self, roots: &[&'a Id]) {
        for root in roots {
            self.discover(root, 0);
        }

        let mut expanded = 0u64;

        while let Some((id, at)) = self.queue.pop_front() {
            if at >= self.depth {
                // The depth boundary: selected into the frontier, expanded by
                // nobody, and never truncation (§7.2, DEC-013).
                self.frontier.push(id);
                continue;
            }

            if expanded == self.budget {
                // A permitted expansion was declined, so the declined entity and
                // everything else still due for expansion are the frontier, in
                // discovery order (DEC-013, DEC-015).
                self.exhausted = true;
                self.frontier.push(id);
                self.frontier
                    .extend(self.queue.drain(..).map(|(queued, _)| queued));
                break;
            }

            expanded += 1;
            self.expand_outgoing(id, at);
            self.expand_incoming(id, at);
        }
    }

    /// §7.1's outgoing expansion: the requested fields of `id`, entirely, before
    /// any incoming one.
    fn expand_outgoing(&mut self, id: &'a Id, at: u64) {
        let Some(fields) = self.follow else {
            return;
        };

        let store = self.store;
        let expanded = stored_entity(store, id);

        for (field, target) in requested_refs(expanded, fields) {
            match store.get_key_value(target) {
                Some((stored, _)) => self.discover(stored, at + 1),
                None => self.note_dangling(id, field, target),
            }
        }
    }

    /// §7.1's incoming expansion: every stored entity holding a requested-field
    /// reference to `id`, in `Entities` order, which is id order.
    fn expand_incoming(&mut self, id: &'a Id, at: u64) {
        let Some(fields) = self.backlinks else {
            return;
        };

        let store = self.store;

        for (source, entity) in store {
            if points_at(entity, id.as_str(), fields) {
                self.discover(source, at + 1);
            }
        }
        // Incoming reports no dangling reference: the source is stored, and the
        // target is the entity being expanded (§7.2).
    }

    /// Records one discovery, its depth, and its place in the queue, unless the
    /// id is already known — a second encounter neither re-enqueues nor reorders
    /// it (§7.1).
    fn discover(&mut self, id: &'a Id, at: u64) {
        if self.seen.insert(id) {
            self.order.push(id);
            self.depths.insert(id, at);
            self.queue.push_back((id, at));
        }
    }

    /// Records one dangling reference unless its `(source, field, target)` triple
    /// is already known, keeping encounter order (§7.2).
    fn note_dangling(&mut self, source: &'a Id, field: &'a str, target: &'a str) {
        if self.dangling_seen.insert((source, field, target)) {
            self.dangling.push(Dangling {
                source,
                field,
                target,
            });
        }
    }

    fn finish(self) -> Walk<'a> {
        Walk {
            order: self.order,
            depths: self.depths,
            frontier: self.frontier,
            dangling: self.dangling,
            exhausted: self.exhausted,
        }
    }
}

/// The entity `id` names in `store`.
///
/// Every id the walk holds came from the store or was found in it, so a miss is a
/// precondition breach: skipping it would silently corrupt [`Walk::order`].
fn stored_entity<'a>(store: &'a Entities, id: &Id) -> &'a Value {
    store
        .get_key_value(id.as_str())
        .map(|(_, entity)| entity)
        .unwrap_or_else(|| {
            panic!("a walk's ids come from its store, and its roots are resolved before it: {id}")
        })
}

/// The `(label, target)` pairs `fields` asks for, in the order the request gives
/// them.
///
/// [`scan_refs`] reports object keys lexically and array items positionally, so
/// grouping by a named field list is the walker's job: the requested labels are
/// iterated in query order and each one filters the scan (§7.4). A duplicate
/// label collapses into its first occurrence, which never reorders the pairs.
fn requested_refs<'e>(entity: &'e Value, fields: &Fields) -> Vec<(&'e str, &'e str)> {
    match fields {
        Fields::All => scan_refs(entity).collect(),
        Fields::Named(labels) => {
            let mut pairs = Vec::new();
            let mut asked: BTreeSet<&str> = BTreeSet::new();

            for label in labels {
                if !asked.insert(label.as_str()) {
                    continue;
                }

                pairs.extend(scan_refs(entity).filter(|(found, _)| *found == label.as_str()));
            }

            pairs
        }
    }
}

/// Whether `entity` holds a reference to `target` in a field `fields` asks for.
fn points_at(entity: &Value, target: &str, fields: &Fields) -> bool {
    scan_refs(entity).any(|(field, found)| found == target && asks_for(fields, field))
}

/// Whether a traversal asked for the field `label`.
fn asks_for(fields: &Fields, label: &str) -> bool {
    match fields {
        Fields::All => true,
        Fields::Named(labels) => labels.iter().any(|asked| asked == label),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    fn store(entities: &[Value]) -> Entities {
        entities
            .iter()
            .map(|entity| {
                let id = entity["id"]
                    .as_str()
                    .expect("every fixture entity names its id");
                (Id::new(id), entity.clone())
            })
            .collect()
    }

    /// The stored ids for `names`, in the order given: a walk borrows its roots
    /// and its discoveries from the snapshot, never from the test.
    fn stored<'a>(store: &'a Entities, names: &[&str]) -> Vec<&'a Id> {
        names
            .iter()
            .map(|name| {
                store
                    .get_key_value(*name)
                    .expect("the fixture holds every named entity")
                    .0
            })
            .collect()
    }

    fn wildcard() -> Fields {
        Fields::All
    }

    fn named(labels: &[&str]) -> Fields {
        Fields::Named(labels.iter().map(|label| (*label).to_owned()).collect())
    }

    fn ordered<'a>(walk: &'a Walk<'_>) -> Vec<&'a str> {
        walk.order.iter().map(|id| id.as_str()).collect()
    }

    fn depths<'a>(walk: &'a Walk<'_>) -> Vec<(&'a str, u64)> {
        walk.depths
            .iter()
            .map(|(id, depth)| (id.as_str(), *depth))
            .collect()
    }

    fn frontier<'a>(walk: &'a Walk<'_>) -> Vec<&'a str> {
        walk.frontier.iter().map(|id| id.as_str()).collect()
    }

    fn dangling<'a>(walk: &'a Walk<'_>) -> Vec<(&'a str, &'a str, &'a str)> {
        walk.dangling
            .iter()
            .map(|found| (found.source.as_str(), found.field, found.target))
            .collect()
    }

    #[test]
    fn outgoing_expansions_run_before_incoming_and_both_discover() {
        // §7.1: one expansion does outgoing entirely, then incoming, so a root's
        // outgoing target is discovered before the sources that point back at it.
        let store = store(&[
            json!({"id": "a", "next": {"id": "b"}}),
            json!({"id": "b"}),
            json!({"id": "z", "next": {"id": "a"}}),
        ]);
        let roots = stored(&store, &["a"]);

        let walked = walk(
            &store,
            &roots,
            Some(&wildcard()),
            Some(&wildcard()),
            10,
            u64::MAX,
        );

        assert_eq!(ordered(&walked), ["a", "b", "z"]);
        assert_eq!(depths(&walked), [("a", 0), ("b", 1), ("z", 1)]);
        assert!(frontier(&walked).is_empty());
        assert!(dangling(&walked).is_empty());
        assert!(!walked.exhausted);
    }

    #[test]
    fn a_named_field_discovers_and_reports_only_what_it_names() {
        // §7.1: a requested label is the only thing a walk looks at, so an
        // unrequested reference is neither discovered nor dangling.
        let store = store(&[
            json!({"id": "a", "link": {"id": "b"}, "note": {"id": "c"}, "other": {"id": "gone"}}),
            json!({"id": "b"}),
            json!({"id": "c"}),
        ]);
        let roots = stored(&store, &["a"]);

        let walked = walk(&store, &roots, Some(&named(&["note"])), None, 10, u64::MAX);

        assert_eq!(ordered(&walked), ["a", "c"]);
        assert_eq!(depths(&walked), [("a", 0), ("c", 1)]);
        assert!(dangling(&walked).is_empty());
    }

    #[test]
    fn named_fields_are_grouped_in_query_order_not_the_scans_lexical_order() {
        // §7.4: `scan_refs` reports object keys lexically, so the walker groups
        // by the requested label list instead of trusting the scan's order.
        let store = store(&[
            json!({"id": "a", "zeta": {"id": "t1"}, "alpha": {"id": "t2"}}),
            json!({"id": "t1"}),
            json!({"id": "t2"}),
        ]);
        let roots = stored(&store, &["a"]);

        let walked = walk(
            &store,
            &roots,
            Some(&named(&["zeta", "alpha", "zeta"])),
            None,
            10,
            u64::MAX,
        );

        assert_eq!(ordered(&walked), ["a", "t1", "t2"]);
        assert_eq!(depths(&walked), [("a", 0), ("t1", 1), ("t2", 1)]);
    }

    #[test]
    fn backlinks_discover_sources_by_the_named_field_only() {
        // §7.1/§7.4: an incoming expansion keeps a source only when the named
        // field points at the expanded entity; sources are discovered in
        // `Entities` order, which is id order.
        let store = store(&[
            json!({"id": "t"}),
            json!({"id": "s1", "link": {"id": "t"}, "memo": {"id": "t"}}),
            json!({"id": "s2", "memo": {"id": "t"}}),
            json!({"id": "s3", "link": {"id": "t"}, "memo": {"id": "t"}}),
        ]);
        let roots = stored(&store, &["t"]);

        let linked = walk(&store, &roots, None, Some(&named(&["link"])), 10, u64::MAX);
        assert_eq!(ordered(&linked), ["t", "s1", "s3"]);
        assert_eq!(depths(&linked), [("s1", 1), ("s3", 1), ("t", 0)]);

        let memoed = walk(&store, &roots, None, Some(&named(&["memo"])), 10, u64::MAX);
        assert_eq!(ordered(&memoed), ["t", "s1", "s2", "s3"]);
        assert!(dangling(&memoed).is_empty());
    }

    #[test]
    fn a_cycle_terminates_with_each_entity_expanded_once() {
        // §7.1: an entity is expanded at most once, so a cycle terminates.
        let store = store(&[
            json!({"id": "a", "next": {"id": "b"}}),
            json!({"id": "b", "next": {"id": "a"}}),
        ]);
        let roots = stored(&store, &["a"]);

        let walked = walk(
            &store,
            &roots,
            Some(&wildcard()),
            Some(&wildcard()),
            10,
            u64::MAX,
        );

        assert_eq!(ordered(&walked), ["a", "b"]);
        assert_eq!(depths(&walked), [("a", 0), ("b", 1)]);
        assert!(frontier(&walked).is_empty());
        assert!(!walked.exhausted);
    }

    #[test]
    fn a_self_reference_is_discovered_once_and_does_not_requeue() {
        // §7.1: a self-reference is its own second encounter, so the seen set
        // absorbs it: `a → a` neither requeues `a` nor re-expands it.
        let store = store(&[json!({"id": "a", "self": {"id": "a"}})]);
        let roots = stored(&store, &["a"]);

        let walked = walk(
            &store,
            &roots,
            Some(&wildcard()),
            Some(&wildcard()),
            10,
            u64::MAX,
        );

        assert_eq!(ordered(&walked), ["a"]);
        assert_eq!(depths(&walked), [("a", 0)]);
        assert!(frontier(&walked).is_empty());
        assert!(dangling(&walked).is_empty());
        assert!(!walked.exhausted);
    }

    #[test]
    fn repeated_roots_and_a_second_discovery_keep_the_first_occurrence() {
        // §7.1: root dedupe keeps the first occurrence in `roots` order, and a
        // second encounter with a discovered entity does not enqueue it again.
        let store = store(&[
            json!({"id": "a", "next": {"id": "c"}}),
            json!({"id": "b", "next": {"id": "c"}}),
            json!({"id": "c"}),
        ]);
        let roots = stored(&store, &["b", "a", "b", "a"]);

        let walked = walk(&store, &roots, Some(&wildcard()), None, 10, u64::MAX);

        assert_eq!(ordered(&walked), ["b", "a", "c"]);
        assert_eq!(depths(&walked), [("a", 0), ("b", 0), ("c", 1)]);
        assert!(frontier(&walked).is_empty());
    }

    #[test]
    fn a_repeated_dangling_triple_is_recorded_once() {
        // §7.2: dangling references dedupe by the `(source, field, target)`
        // triple, so the same target twice in one field is one dangling
        // reference while the same target in another field is a second one.
        let store = store(&[json!({
            "id": "a",
            "list": [{"id": "gone"}, {"id": "gone"}],
            "other": {"id": "gone"},
        })]);
        let roots = stored(&store, &["a"]);

        let walked = walk(&store, &roots, Some(&wildcard()), None, 10, u64::MAX);

        assert_eq!(ordered(&walked), ["a"]);
        assert_eq!(
            dangling(&walked),
            [("a", "list", "gone"), ("a", "other", "gone")]
        );
    }

    #[test]
    fn discovery_is_roots_first_in_root_order_then_breadth_first() {
        // §7.1: the queue is breadth-first, so `r2`'s target precedes `r1`'s
        // chain — a depth-first walk would have gone `r1`, `a1`, `a2`.
        let store = store(&[
            json!({"id": "r1", "next": {"id": "a1"}}),
            json!({"id": "a1", "next": {"id": "a2"}}),
            json!({"id": "a2"}),
            json!({"id": "r2", "next": {"id": "b1"}}),
            json!({"id": "b1"}),
        ]);
        let roots = stored(&store, &["r2", "r1"]);

        let walked = walk(&store, &roots, Some(&wildcard()), None, 10, u64::MAX);

        assert_eq!(ordered(&walked), ["r2", "r1", "b1", "a1", "a2"]);
        assert_eq!(
            depths(&walked),
            [("a1", 1), ("a2", 2), ("b1", 1), ("r1", 0), ("r2", 0)]
        );
        assert!(frontier(&walked).is_empty());
    }

    #[test]
    fn the_depth_boundary_is_frontier_not_truncation() {
        // §7.1/§7.2/DEC-013: a node at the depth bound is selected into the
        // frontier and expanded by nobody, which is a depth boundary, not
        // truncation: `exhausted` stays false at every bound.
        let store = store(&[
            json!({"id": "a", "next": {"id": "b"}}),
            json!({"id": "b", "next": {"id": "c"}}),
            json!({"id": "c"}),
        ]);
        let roots = stored(&store, &["a"]);

        let cut_at_root = walk(&store, &roots, Some(&wildcard()), None, 0, u64::MAX);
        assert_eq!(ordered(&cut_at_root), ["a"]);
        assert_eq!(depths(&cut_at_root), [("a", 0)]);
        assert_eq!(frontier(&cut_at_root), ["a"]);
        assert!(!cut_at_root.exhausted);

        let cut_after_roots = walk(&store, &roots, Some(&wildcard()), None, 1, u64::MAX);
        assert_eq!(ordered(&cut_after_roots), ["a", "b"]);
        assert_eq!(depths(&cut_after_roots), [("a", 0), ("b", 1)]);
        assert_eq!(frontier(&cut_after_roots), ["b"]);
        assert!(!cut_after_roots.exhausted);
    }

    #[test]
    fn nested_and_embedded_references_are_discovered_exactly_as_scan_refs_reports_them() {
        // §12 row 1: one reference interpretation. A reference nested inside
        // `outer` and entity nodes inside a list are discovered as `scan_refs`
        // reports them, and the dangling field is the top-level field name —
        // `box`, never `deep` — which is what proves there is one interpretation.
        let store = store(&[
            json!({"id": "a", "outer": {"inner": {"id": "target"}}}),
            json!({"id": "b", "list": [{"id": "t2"}]}),
            json!({"id": "c", "box": {"deep": {"id": "gone"}}}),
            json!({"id": "target"}),
            json!({"id": "t2"}),
        ]);
        let roots = stored(&store, &["a", "b", "c"]);

        let walked = walk(&store, &roots, Some(&wildcard()), None, 1, u64::MAX);

        assert_eq!(ordered(&walked), ["a", "b", "c", "target", "t2"]);
        assert_eq!(
            depths(&walked),
            [("a", 0), ("b", 0), ("c", 0), ("t2", 1), ("target", 1)]
        );
        assert_eq!(frontier(&walked), ["target", "t2"]);
        assert_eq!(dangling(&walked), [("c", "box", "gone")]);
        assert!(!walked.exhausted);
    }

    #[test]
    fn an_unbounded_budget_never_exhausts_the_walk_or_cuts_the_queue() {
        // §7.3: PHASE-01 passes `u64::MAX`, so no permitted expansion is ever
        // declined and the walk runs to its depth bound.
        let store = store(&[
            json!({"id": "a", "next": {"id": "b"}}),
            json!({"id": "b", "next": {"id": "c"}}),
            json!({"id": "c", "next": {"id": "d"}}),
            json!({"id": "d"}),
        ]);
        let roots = stored(&store, &["a"]);

        let walked = walk(&store, &roots, Some(&wildcard()), None, 10, u64::MAX);

        assert_eq!(ordered(&walked), ["a", "b", "c", "d"]);
        assert_eq!(depths(&walked), [("a", 0), ("b", 1), ("c", 2), ("d", 3)]);
        assert!(frontier(&walked).is_empty());
        assert!(!walked.exhausted);
    }

    #[test]
    fn a_budget_equal_to_the_expansions_completes_the_walk() {
        // §7.3/DEC-015: the check runs before the expansion, so a walk that
        // empties its queue with exactly `budget` expansions is complete.
        let store = store(&[
            json!({"id": "a", "next": {"id": "b"}}),
            json!({"id": "b", "next": {"id": "c"}}),
            json!({"id": "c"}),
        ]);
        let roots = stored(&store, &["a"]);

        let walked = walk(&store, &roots, Some(&wildcard()), None, 10, 3);

        assert_eq!(ordered(&walked), ["a", "b", "c"]);
        assert_eq!(depths(&walked), [("a", 0), ("b", 1), ("c", 2)]);
        assert!(frontier(&walked).is_empty());
        assert!(!walked.exhausted);
    }

    #[test]
    fn one_expansion_short_declines_a_permitted_expansion_and_frontiers_the_queues_rest() {
        // §7.3/DEC-013: the declined node and everything still queued become the
        // frontier, in discovery order, and only a declined expansion sets
        // `exhausted`.
        let store = store(&[
            json!({"id": "a", "next": [{"id": "b"}, {"id": "c"}]}),
            json!({"id": "b"}),
            json!({"id": "c"}),
        ]);
        let roots = stored(&store, &["a"]);

        let walked = walk(&store, &roots, Some(&wildcard()), None, 10, 1);

        assert_eq!(ordered(&walked), ["a", "b", "c"]);
        assert_eq!(depths(&walked), [("a", 0), ("b", 1), ("c", 1)]);
        assert_eq!(frontier(&walked), ["b", "c"]);
        assert!(walked.exhausted);
    }
}
