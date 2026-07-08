//! Tests for the graph module. Uses an inline JSON fixture (no dev-deps).

use super::*;
use crate::graph::render::{render, render_spine, render_tree};
use crate::models::{CreateScriptRequest, Operation, Predicate};

/// A 5-node script with a cycle and an unreachable node:
///   entry /start -> 0
///   0 greeting  --[always]--> 1 menu
///   1 menu      --["about"]--> 2 about
///               --["quit"]-->  3 end
///               --["again"]--> 1 menu   (self-cycle)
///   4 orphan    (unreachable)
const FIXTURE_JSON: &str = r#"{
  "desc": "test fixture",
  "entries": [
    { "key": "start", "start": 0 }
  ],
  "nodes": [
    {
      "state": 0, "title": "greeting",
      "messages": [{ "text": "Hi! Choose:" }],
      "edges": [
        { "predicate": { "type": "always" }, "to": 1, "operation": "noop" }
      ]
    },
    {
      "state": 1, "title": "menu",
      "messages": [],
      "options": ["about", "quit", "again"],
      "edges": [
        { "predicate": { "type": "exact", "text": "about" }, "to": 2, "operation": "save" },
        { "predicate": { "type": "exact", "text": "quit" },  "to": 3, "operation": "save" },
        { "predicate": { "type": "exact", "text": "again" }, "to": 1, "operation": "noop" }
      ]
    },
    {
      "state": 2, "title": "about",
      "messages": [{ "text": "About us" }],
      "edges": [
        { "predicate": { "type": "always" }, "to": 1, "operation": "noop" }
      ]
    },
    {
      "state": 3, "title": "end",
      "messages": [{ "text": "Bye!" }]
    },
    {
      "state": 4, "title": "orphan",
      "messages": []
    }
  ]
}"#;

fn fixture() -> CreateScriptRequest {
    serde_json::from_str(FIXTURE_JSON).expect("fixture must parse")
}

#[test]
fn parse_fixture() {
    let req = fixture();
    assert_eq!(req.nodes.len(), 5);
    assert_eq!(req.entries.len(), 1);
    // Round-trip through render to catch serde/layout regressions.
    let idx = GraphIndex::new(&req);
    let _ = render(&idx, GraphStyle::Spine);
    let _ = render(&idx, GraphStyle::Tree);
}

#[test]
fn entry_states() {
    let req = fixture();
    let idx = GraphIndex::new(&req);
    assert_eq!(idx.entries(), &[0]);
}

#[test]
fn order_is_stable() {
    let req = fixture();
    let idx = GraphIndex::new(&req);
    // Starts with the entry, then BFS order; orphan at the very end.
    assert_eq!(*idx.order(), vec![0, 1, 2, 3, 4]);
}

#[test]
fn unreachable() {
    let req = fixture();
    let idx = GraphIndex::new(&req);
    assert_eq!(idx.unreachable_states(), vec![4]);
}

#[test]
fn cycle_detection() {
    let req = fixture();
    let idx = GraphIndex::new(&req);
    let cycles = idx.cycle_states();
    // 1 → 1 self-cycle.
    assert!(
        cycles.contains(&1),
        "state 1 must be cyclic, got {:?}",
        cycles
    );
}

#[test]
fn forward_vs_back_edges() {
    let req = fixture();
    let idx = GraphIndex::new(&req);

    // 1 → 2 and 1 → 3 are forward; 1 → 1 is back.
    let back: Vec<i32> = idx.back_edges(1).iter().map(|e| e.to).collect();
    assert_eq!(back, vec![1]);

    let fwd: std::collections::BTreeSet<i32> = idx.forward_edges(1).iter().map(|e| e.to).collect();
    assert!(fwd.contains(&2) && fwd.contains(&3));
    assert!(!fwd.contains(&1));

    // 2 → 1 points back up the spine (2 is after 1) => back edge.
    let back2: Vec<i32> = idx.back_edges(2).iter().map(|e| e.to).collect();
    assert_eq!(back2, vec![1]);
}

#[test]
fn predicate_label_variants() {
    assert_eq!(predicate_label(&Predicate::default()), "always");
    let exact = Predicate::Exact(Box::new(crate::models::ExactPredicate {
        text: "hi".to_string(),
    }));
    assert_eq!(predicate_label(&exact), "\"hi\"");
    let regex = Predicate::Regex(Box::new(crate::models::RegexPredicate {
        pattern: "^[0-9]+$".to_string(),
    }));
    assert_eq!(predicate_label(&regex), "/^[0-9]+$/");
}

#[test]
fn render_spine_snapshot() {
    let req = fixture();
    let idx = GraphIndex::new(&req);
    let out = render_spine(&idx);
    assert!(out.contains("Entries:"));
    assert!(out.contains("-> state 0"), "missing entry marker:\n{}", out);
    assert!(out.contains("[1] menu"), "missing menu node:\n{}", out);
    assert!(out.contains("Unreachable:"), "missing unreachable section");
    assert!(out.contains("[4] orphan"), "missing orphan node");
    // The 1 -> 1 self-cycle should surface as a back-edge arc with ↺.
    assert!(out.contains("1 ↺"), "missing cycle marker:\n{}", out);
}

#[test]
fn render_tree_no_infinite_recursion() {
    let req = fixture();
    let idx = GraphIndex::new(&req);
    // If the self-cycle caused infinite recursion this would overflow.
    let out = render_tree(&idx);
    assert!(out.contains("1 · menu"), "missing menu node:\n{}", out);
    assert!(out.contains("1 ↺"), "missing cycle marker:\n{}", out);
}

#[test]
fn render_tree_snapshot() {
    let req = fixture();
    let idx = GraphIndex::new(&req);
    let out = render_tree(&idx);
    // Root and at least one connector.
    assert!(out.contains("0 · greeting"));
    assert!(
        out.contains("└──") || out.contains("├──"),
        "missing tree connectors:\n{}",
        out
    );
    assert!(out.contains("Unreachable:"));
}

#[test]
fn operation_serializes_lowercase() {
    assert_eq!(serde_json::to_string(&Operation::Noop).unwrap(), "\"noop\"");
    assert_eq!(serde_json::to_string(&Operation::Save).unwrap(), "\"save\"");
    assert_eq!(
        serde_json::to_string(&Operation::Append).unwrap(),
        "\"append\""
    );
}

#[test]
fn self_cycle_detected_in_larger_loop() {
    // 0 -> 1 -> 2 -> 0  (3-cycle) plus a tail 0 -> 3.
    let req: CreateScriptRequest = serde_json::from_str(
        r#"{
      "desc": "loop",
      "entries": [{ "key": "start", "start": 0 }],
      "nodes": [
        { "state": 0, "title": "a", "messages": [],
          "edges": [
            { "predicate": {"type":"always"}, "to": 1, "operation": "noop" },
            { "predicate": {"type":"always"}, "to": 3, "operation": "noop" }
          ] },
        { "state": 1, "title": "b", "messages": [],
          "edges": [ { "predicate": {"type":"always"}, "to": 2, "operation": "noop" } ] },
        { "state": 2, "title": "c", "messages": [],
          "edges": [ { "predicate": {"type":"always"}, "to": 0, "operation": "noop" } ] },
        { "state": 3, "title": "d", "messages": [] }
      ]
    }"#,
    )
    .unwrap();
    let idx = GraphIndex::new(&req);
    let cycles = idx.cycle_states();
    for s in [0, 1, 2] {
        assert!(cycles.contains(&s), "state {} must be cyclic", s);
    }
    assert!(!cycles.contains(&3), "state 3 must NOT be cyclic");
}
