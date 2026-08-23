//! Plain-text renderers for a [`super::GraphIndex`].
//!
//! Both renderers are pure functions returning a `String`, which makes them
//! trivial to snapshot-test and pipe-friendly in `--plain` mode.

use crate::graph::{GraphIndex, GraphStyle, predicate_label};

/// Render the graph with the given [`GraphStyle`].
pub fn render(idx: &GraphIndex<'_>, style: GraphStyle) -> String {
    match style {
        GraphStyle::Spine => render_spine(idx),
        GraphStyle::Tree => render_tree(idx),
    }
}

/// Spine layout: nodes stacked along a vertical axis, forward edges as the
/// spine connector, back edges (cycles) drawn as side arcs labelled with the
/// predicate and `↺`.
pub fn render_spine(idx: &GraphIndex<'_>) -> String {
    let mut out = String::new();

    if idx.entries().is_empty() {
        out.push_str("Entries: (none)\n\n");
    } else {
        out.push_str("Entries:\n");
        for &s in idx.entries() {
            out.push_str(&format!("  -> state {}\n", s));
        }
        out.push('\n');
    }

    let cyclic = idx.cycle_states();
    let unreachable: std::collections::BTreeSet<i32> =
        idx.unreachable_states().into_iter().collect();

    let reachable: Vec<i32> = idx
        .order()
        .iter()
        .copied()
        .filter(|s| !unreachable.contains(s))
        .collect();

    out.push_str(&render_spine_section(idx, &reachable, &cyclic, ""));

    if !unreachable.is_empty() {
        out.push_str("\nUnreachable:\n");
        let orphan: Vec<i32> = unreachable.into_iter().collect();
        out.push_str(&render_spine_section(idx, &orphan, &cyclic, "  "));
    }

    out
}

fn render_spine_section(
    idx: &GraphIndex<'_>,
    states: &[i32],
    cyclic: &std::collections::BTreeSet<i32>,
    indent: &str,
) -> String {
    let mut out = String::new();
    let last = states.len().saturating_sub(1);

    for (i, &s) in states.iter().enumerate() {
        let Some(node) = idx.node(s) else {
            continue;
        };
        let marker = if cyclic.contains(&s) { " ↺" } else { "" };
        out.push_str(&format!("{indent}[{}] {}{}\n", s, node.title, marker));

        let next = states.get(i + 1).copied();
        let forwards = idx.forward_edges(s);

        // Edges to the immediately-following spine node are absorbed into the
        // connector; only annotate when there's a single such edge.
        let spine_edge_count = forwards.iter().filter(|e| Some(e.to) == next).count();
        if spine_edge_count == 1 {
            let e = forwards
                .iter()
                .find(|e| Some(e.to) == next)
                .expect("counted above");
            out.push_str(&format!(
                "{indent}  │  [{}]\n",
                predicate_label(&e.predicate)
            ));
        }

        // Non-adjacent forward edges + back edges get explicit arrows.
        for e in &forwards {
            if Some(e.to) != next || spine_edge_count != 1 {
                out.push_str(&format!(
                    "{indent}  └─[{}]→ {}\n",
                    predicate_label(&e.predicate),
                    e.to
                ));
            }
        }
        for e in idx.back_edges(s) {
            out.push_str(&format!(
                "{indent}  ╭─[{}]→ {} ↺\n",
                predicate_label(&e.predicate),
                e.to
            ));
        }

        if i < last {
            out.push_str(&format!("{indent}  │\n"));
        }
    }

    out
}

/// Tree layout: DFS from each entry, `├──`/`└──` connectors, back edges shown
/// as `[pred]→ N ↺` without recursing into them (so cycles never blow the
/// stack).
pub fn render_tree(idx: &GraphIndex<'_>) -> String {
    let mut out = String::new();

    if idx.entries().is_empty() {
        out.push_str("(no entry points)\n");
    } else {
        for &root in idx.entries() {
            let mut visited = std::collections::BTreeSet::new();
            // Root: no connector, no incoming label.
            walk_tree(idx, root, "", None, &mut visited, &mut out);
            out.push('\n');
        }
    }

    let unreachable = idx.unreachable_states();
    if !unreachable.is_empty() {
        out.push_str("Unreachable:\n");
        for &s in &unreachable {
            let title = idx.node(s).map(|n| n.title.as_str()).unwrap_or("?");
            out.push_str(&format!("  {} · {}\n", s, title));
        }
    }

    out
}

fn walk_tree(
    idx: &GraphIndex<'_>,
    state: i32,
    prefix: &str,
    incoming: Option<&str>,
    visited: &mut std::collections::BTreeSet<i32>,
    out: &mut String,
) {
    let node = match idx.node(state) {
        Some(n) => n,
        None => {
            out.push_str(&format!("{prefix}── ? {}\n", state));
            return;
        }
    };

    match incoming {
        Some(label) => out.push_str(&format!(
            "{prefix}── [{}]→ {} · {}\n",
            label, state, node.title
        )),
        None => out.push_str(&format!("{} · {}\n", state, node.title)),
    }

    // Expand each node at most once to avoid infinite recursion on cycles.
    if !visited.insert(state) {
        return;
    }

    let edges = idx.edges(state);
    let last_idx = edges.len().saturating_sub(1);
    for (i, e) in edges.iter().enumerate() {
        let is_last = i == last_idx;
        let branch = if is_last { "└──" } else { "├──" };
        let label = predicate_label(&e.predicate);
        let _ = branch; // connector is drawn by the recursive call

        if visited.contains(&e.to) {
            // Already-expanded target: leaf with ↺, no recursion.
            out.push_str(&format!(
                "{prefix}   {} [{}]→ {} ↺\n",
                if is_last { "└──" } else { "├──" },
                label,
                e.to
            ));
        } else {
            let child_prefix = format!("{}{}", prefix, if is_last { "   " } else { "│  " });
            walk_tree(idx, e.to, &child_prefix, Some(&label), visited, out);
        }
    }
}
