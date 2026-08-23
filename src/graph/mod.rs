//! State-graph analysis and rendering for bot scripts.
//!
//! A [`crate::models::Script`] / [`crate::models::CreateScriptRequest`] is a
//! directed graph: nodes are states, edges (`to`) are transitions, entries are
//! roots. This module indexes the graph and classifies its edges without
//! touching any I/O, so everything here is trivially unit-testable.

pub mod render;
pub mod tui;

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::models::{CreateScriptRequest, Edge, Entry, Node, Predicate, Script};

/// Anything that exposes the graph structure of a script.
///
/// Implemented for the two shapes that carry a graph: the request sent to the
/// API ([`CreateScriptRequest`]) and the full entity returned by it
/// ([`Script`]). Rendering/analysis work on the trait so both are supported
/// from a single code path.
pub trait StateGraph {
    fn nodes(&self) -> &[Node];
    fn entries(&self) -> &[Entry];
}

impl StateGraph for CreateScriptRequest {
    fn nodes(&self) -> &[Node] {
        &self.nodes
    }
    fn entries(&self) -> &[Entry] {
        &self.entries
    }
}

impl StateGraph for Script {
    fn nodes(&self) -> &[Node] {
        &self.nodes
    }
    fn entries(&self) -> &[Entry] {
        &self.entries
    }
}

/// How to lay the graph out for rendering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GraphStyle {
    /// Nodes stacked along a vertical spine, cycle edges drawn as side arcs.
    Spine,
    /// Classic DFS tree with `├──`/`└──` connectors.
    Tree,
}

/// Pre-computed, owning view over a script's graph.
///
/// Borrows the source graph for the whole of its lifetime — building it is
/// cheap (a single BFS) and all lookups afterwards are `O(log n)`.
#[derive(Debug)]
pub struct GraphIndex<'a> {
    by_state: BTreeMap<i32, &'a Node>,
    /// Entry states sorted by their entry `key` for stable ordering.
    entries: Vec<i32>,
    /// Entries as `(key, start)` sorted by key — for entry-point pickers.
    entry_list: Vec<(String, i32)>,
    /// Vertical layout order: entry roots first, then BFS-reachable, then
    /// unreachable nodes. This is the "spine" down which nodes are stacked.
    order: Vec<i32>,
    /// Position of each state in [`order`]; used to tell forward from back
    /// edges (a back edge points to an ancestor => it closes a cycle).
    pos: BTreeMap<i32, usize>,
}

impl<'a> GraphIndex<'a> {
    /// Build the index from any graph source.
    pub fn new<G: StateGraph + 'a>(graph: &'a G) -> Self {
        let by_state: BTreeMap<i32, &Node> = graph.nodes().iter().map(|n| (n.state, n)).collect();

        // Entries as (key, start), keeping only those whose start resolves to a
        // known node, sorted by key for deterministic output.
        let mut entry_list: Vec<(String, i32)> = graph
            .entries()
            .iter()
            .map(|e| (e.key.clone(), e.start))
            .filter(|(_, s)| by_state.contains_key(s))
            .collect();
        entry_list.sort_by(|a, b| a.0.cmp(&b.0));
        entry_list.dedup_by(|a, b| a.0 == b.0);

        // Entry root states (deduplicated) sorted by key.
        let entries: Vec<i32> = entry_list.iter().map(|(_, s)| *s).collect();
        let mut entries_sorted = entries.clone();
        entries_sorted.sort();
        entries_sorted.dedup();

        // BFS from the entry roots to get a stable reachable order.
        let mut seen: BTreeSet<i32> = entries_sorted.iter().copied().collect();
        let mut order: Vec<i32> = entries_sorted.clone();
        let mut queue: VecDeque<i32> = entries_sorted.iter().copied().collect();
        while let Some(s) = queue.pop_front() {
            let Some(node) = by_state.get(&s) else {
                continue;
            };
            for to in edges_of(node).iter().map(|e| e.to) {
                if seen.insert(to) {
                    order.push(to);
                    queue.push_back(to);
                }
            }
        }

        // Unreachable nodes go last, sorted by state id for stability.
        let mut unreachable: Vec<i32> = by_state
            .keys()
            .copied()
            .filter(|s| !seen.contains(s))
            .collect();
        unreachable.sort();
        order.extend(unreachable);

        let pos = order.iter().enumerate().map(|(i, s)| (*s, i)).collect();

        Self {
            by_state,
            entries,
            entry_list,
            order,
            pos,
        }
    }

    /// Entry (root) states in display order (deduplicated, sorted by state id).
    pub fn entries(&self) -> &[i32] {
        &self.entries
    }

    /// Entries as `(key, start)` pairs, sorted by key — for entry-point pickers.
    pub fn entry_list(&self) -> &[(String, i32)] {
        &self.entry_list
    }

    /// The vertical layout order — the spine.
    pub fn order(&self) -> &[i32] {
        &self.order
    }

    pub fn node(&self, state: i32) -> Option<&Node> {
        self.by_state.get(&state).copied()
    }

    /// Outgoing edges of `state` (empty if none / unknown).
    pub fn edges(&self, state: i32) -> Vec<&Edge> {
        self.node(state)
            .map(|n| edges_of(n).iter().collect())
            .unwrap_or_default()
    }

    /// Edges that point "down" the spine (to a later node that is not an
    /// ancestor and not a self-loop). These are drawn as normal connectors.
    pub fn forward_edges(&self, state: i32) -> Vec<&Edge> {
        let from_pos = self.pos.get(&state).copied();
        self.edges(state)
            .into_iter()
            .filter(|e| {
                // A self-loop (state == to) is a cycle, never forward.
                if e.to == state {
                    return false;
                }
                // An edge is "forward" unless it points to an earlier position
                // on the spine, which makes it a back edge (cycle).
                match (from_pos, self.pos.get(&e.to)) {
                    (Some(a), Some(b)) => a <= *b,
                    // Unknown target can't be a spine cycle; treat as forward.
                    _ => true,
                }
            })
            .collect()
    }

    /// Edges that close a cycle by pointing back up the spine, or self-loops.
    pub fn back_edges(&self, state: i32) -> Vec<&Edge> {
        let from_pos = self.pos.get(&state).copied();
        self.edges(state)
            .into_iter()
            .filter(|e| {
                if e.to == state {
                    return true; // self-loop
                }
                match (from_pos, self.pos.get(&e.to)) {
                    (Some(a), Some(b)) => a > *b,
                    _ => false,
                }
            })
            .collect()
    }

    /// States that participate in a cycle, found via DFS with a recursion
    /// stack. A back edge (to a node currently on the stack) marks every node
    /// between the target and the current node as cyclic.
    pub fn cycle_states(&self) -> BTreeSet<i32> {
        let mut cyclic = BTreeSet::new();
        let mut color: BTreeMap<i32, u8> = BTreeMap::new(); // 0=white,1=gray,2=black
        let mut stack: Vec<i32> = Vec::new();

        for &root in self.order.iter() {
            if color.get(&root).copied().unwrap_or(0) != 0 {
                continue;
            }
            self.dfs_cycles(root, &mut color, &mut stack, &mut cyclic);
        }
        cyclic
    }

    fn dfs_cycles(
        &self,
        state: i32,
        color: &mut BTreeMap<i32, u8>,
        stack: &mut Vec<i32>,
        cyclic: &mut BTreeSet<i32>,
    ) {
        color.insert(state, 1);
        stack.push(state);

        for to in self.edges(state).into_iter().map(|e| e.to) {
            match color.get(&to).copied().unwrap_or(0) {
                1 => {
                    // Back edge: mark every node on the stack from `to` down.
                    if let Some(start) = stack.iter().position(|&s| s == to) {
                        for &s in &stack[start..] {
                            cyclic.insert(s);
                        }
                    }
                }
                0 => self.dfs_cycles(to, color, stack, cyclic),
                _ => {}
            }
        }

        stack.pop();
        color.insert(state, 2);
    }

    /// States not reachable from any entry.
    pub fn unreachable_states(&self) -> Vec<i32> {
        let reachable: BTreeSet<i32> = self.entries.iter().copied().collect();
        let mut reach = reachable.clone();
        let mut queue: VecDeque<i32> = reach.iter().copied().collect();
        while let Some(s) = queue.pop_front() {
            for to in self.edges(s).into_iter().map(|e| e.to) {
                if reach.insert(to) {
                    queue.push_back(to);
                }
            }
        }
        self.order
            .iter()
            .copied()
            .filter(|s| !reach.contains(s))
            .collect()
    }
}

/// Outgoing edges of a node, handling the `Option<Vec>` shape from the model.
fn edges_of(node: &Node) -> &[Edge] {
    node.edges.as_deref().unwrap_or(&[])
}

/// A short, human-readable label for an edge's predicate.
///
/// `Always` → `always`, `Exact("foo")` → `"foo"`, `Regex(...)` → `/re/`.
/// Used by both renderers and the TUI detail panel, so it lives here.
pub fn predicate_label(p: &Predicate) -> String {
    match p {
        Predicate::Always(_) => "always".to_string(),
        Predicate::Exact(e) => format!("\"{}\"", e.text),
        Predicate::Regex(r) => format!("/{}/", r.pattern),
        Predicate::Unknown(v) => v.to_string(),
    }
}

#[cfg(test)]
mod tests;
