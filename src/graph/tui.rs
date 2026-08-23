//! Interactive full-screen TUI for walking a script's state graph.
//!
//! Built on `ratatui` + `crossterm`. The TUI walks the graph like a user
//! following the bot: an entry-point picker leads into a step-by-step traversal.
//!
//! Layout (Walk mode):
//! - **Left pane (Tree)** — a local, `tree`-command-style view centered on the
//!   current node: current + its descendants to a few levels, OR (when the
//!   current node is itself an entry) all entries as sibling roots with the
//!   current one expanded. `Enter` on any tree node teleports the walk there and
//!   resets the path.
//! - **Right column** — `Path` (the recent walk trail), `Current` (node
//!   details), `Children` (outgoing transitions, stepped into with `Enter`).

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Stdout};

use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use crate::graph::{GraphIndex, GraphStyle, predicate_label};
use crate::models::Node;

type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Run the interactive TUI. Restores the terminal on exit (even on error).
pub fn run(idx: &GraphIndex<'_>, _style: GraphStyle) -> io::Result<()> {
    let mut app = App::new(idx);
    let mut terminal = setup_terminal()?;
    let result = app.loop_(&mut terminal);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> io::Result<Tui> {
    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(io::stdout());
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Tui) -> io::Result<()> {
    disable_raw_mode()?;
    terminal.backend_mut().execute(LeaveAlternateScreen)?;
    Ok(())
}

/// How many of the most recent steps of the walk to show in the Path pane.
const PATH_WINDOW: usize = 4;
/// Max descendant depth to expand under the current node in the tree pane.
const TREE_CHILD_DEPTH: usize = 3;

/// Which screen the app is on.
#[derive(Debug)]
enum Mode {
    /// Entry-point picker shown at startup and after `r`/restart.
    PickEntry { sel: ListState },
    /// Step-by-step graph walk.
    Walk,
}

/// Which pane holds focus during a [`Mode::Walk`]. `Tab` cycles in this order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Focus {
    Tree,
    Path,
    Children,
}

impl Focus {
    fn next(self) -> Self {
        match self {
            Focus::Tree => Focus::Path,
            Focus::Path => Focus::Children,
            Focus::Children => Focus::Tree,
        }
    }
}

/// A single reachable target from a node, with the predicates that lead to it
/// grouped together.
#[derive(Debug)]
struct ChildInfo {
    to: i32,
    title: String,
    labels: Vec<String>,
    /// Target is already on the current walk path → returning to it (`↺`).
    is_revisit: bool,
}

/// One flattened line of the tree pane.
#[derive(Debug)]
struct TreeLine {
    /// State id this line refers to (for selection / teleport).
    state: i32,
    /// Pre-rendered connector prefix (`│  `, `├── `, …) for indentation.
    prefix: String,
    /// Whether this line is the current walk node.
    is_here: bool,
    /// Target already expanded somewhere above in the tree (cycle leaf).
    is_cycle: bool,
}

struct App<'a> {
    idx: &'a GraphIndex<'a>,
    mode: Mode,
    /// Walk history; `path.last()` is the current node.
    path: Vec<i32>,
    /// Key of the entry the current walk started from (for titles).
    entry_key: Option<String>,
    child_state: ListState,
    tree_state: ListState,
    /// Selection within the Path pane (independent cursor for trimming).
    path_state: ListState,
    focus: Focus,
    cyclic: BTreeSet<i32>,
    unreachable: BTreeSet<i32>,
}

impl<'a> App<'a> {
    fn new(idx: &'a GraphIndex<'_>) -> Self {
        let mut sel = ListState::default();
        if !idx.entry_list().is_empty() {
            sel.select(Some(0));
        }
        Self {
            idx,
            mode: Mode::PickEntry { sel },
            path: Vec::new(),
            entry_key: None,
            child_state: ListState::default(),
            tree_state: ListState::default(),
            path_state: ListState::default(),
            focus: Focus::Children,
            cyclic: idx.cycle_states(),
            unreachable: idx.unreachable_states().into_iter().collect(),
        }
    }

    fn current(&self) -> Option<i32> {
        self.path.last().copied()
    }

    /// Children (unique edge targets) of `state`.
    fn build_children(&self, state: i32) -> Vec<ChildInfo> {
        let mut order: Vec<i32> = Vec::new();
        let mut by_to: BTreeMap<i32, Vec<String>> = BTreeMap::new();
        for e in self.idx.edges(state) {
            let label = predicate_label(&e.predicate);
            by_to.entry(e.to).or_default().push(label);
            if !order.contains(&e.to) {
                order.push(e.to);
            }
        }
        order
            .into_iter()
            .map(|to| ChildInfo {
                to,
                title: self.idx.node(to).map(|n| n.title.clone()).unwrap_or_default(),
                labels: by_to.remove(&to).unwrap_or_default(),
                is_revisit: self.path.contains(&to),
            })
            .collect()
    }

    /// Begin a walk from the given entry.
    fn start_walk(&mut self, key: String, start: i32) {
        self.path.clear();
        self.path.push(start);
        self.entry_key = Some(key);
        self.child_state.select(Some(0));
        self.path_state.select(Some(0));
        self.focus = Focus::Children;
        self.reset_tree_cursor();
        self.mode = Mode::Walk;
    }

    /// Step into the selected child (Children pane).
    fn enter_child(&mut self, to: i32) {
        self.path.push(to);
        self.child_state.select(Some(0));
        self.reset_tree_cursor();
    }

    /// Teleport: reset the path so that `state` is the only/current node.
    fn teleport(&mut self, state: i32) {
        self.path.clear();
        self.path.push(state);
        self.child_state.select(Some(0));
        self.path_state.select(Some(0));
        self.reset_tree_cursor();
    }

    /// Trim the path up to and including the selected Path-cursor point.
    fn trim_path_to(&mut self, index: usize) {
        if self.path.len() <= 1 || index >= self.path.len() {
            return;
        }
        self.path.truncate(index + 1);
        self.child_state.select(Some(0));
        let new_sel = self.path.len().saturating_sub(1);
        self.path_state.select(Some(new_sel));
        self.reset_tree_cursor();
    }

    /// Step back one node, never below the entry root.
    fn go_back(&mut self) {
        if self.path.len() <= 1 {
            return;
        }
        self.path.pop();
        self.child_state.select(Some(0));
        let new_sel = self.path.len().saturating_sub(1);
        self.path_state.select(Some(new_sel));
        self.reset_tree_cursor();
    }

    /// Back to the entry-point picker.
    fn restart(&mut self) {
        self.path.clear();
        self.entry_key = None;
        let mut sel = ListState::default();
        if !self.idx.entry_list().is_empty() {
            sel.select(Some(0));
        }
        self.mode = Mode::PickEntry { sel };
    }

    /// Re-center the tree cursor on the current node.
    fn reset_tree_cursor(&mut self) {
        let lines = self.build_tree_lines();
        let here = lines.iter().position(|l| l.is_here);
        self.tree_state.select(here.or(Some(0)));
    }

    fn move_child(&mut self, delta: i32) {
        let Some(cur) = self.current() else {
            return;
        };
        let len = self.build_children(cur).len();
        if len == 0 {
            return;
        }
        let next = clamp(self.child_state.selected().unwrap_or(0) as i32 + delta, len);
        self.child_state.select(Some(next as usize));
    }

    fn move_tree(&mut self, delta: i32) {
        let len = self.build_tree_lines().len();
        if len == 0 {
            return;
        }
        let next = clamp(self.tree_state.selected().unwrap_or(0) as i32 + delta, len);
        self.tree_state.select(Some(next as usize));
    }

    fn move_path(&mut self, delta: i32) {
        let len = self.path.len();
        if len == 0 {
            return;
        }
        let next = clamp(self.path_state.selected().unwrap_or(0) as i32 + delta, len);
        self.path_state.select(Some(next as usize));
    }

    fn move_entry(&mut self, delta: i32) {
        let len = self.idx.entry_list().len();
        if len == 0 {
            return;
        }
        let Mode::PickEntry { sel } = &mut self.mode else {
            return;
        };
        let next = clamp(sel.selected().unwrap_or(0) as i32 + delta, len);
        sel.select(Some(next as usize));
    }

    /// Flatten the focused tree into selectable lines.
    fn build_tree_lines(&self) -> Vec<TreeLine> {
        let Some(cur) = self.current() else {
            return Vec::new();
        };
        let entry_starts: BTreeSet<i32> = self.idx.entries().iter().copied().collect();

        let mut lines = Vec::new();
        let mut visited = BTreeSet::new();

        if entry_starts.contains(&cur) {
            // Current node is an entry: render all entries as sibling roots.
            let roots: Vec<i32> = self.idx.entry_list().iter().map(|(_, s)| *s).collect();
            let last_root = roots.len().saturating_sub(1);
            for (ri, root) in roots.iter().enumerate() {
                let is_last_root = ri == last_root;
                let child_prefix = if is_last_root { "   " } else { "│  " };
                // Root line itself.
                lines.push(TreeLine {
                    state: *root,
                    prefix: String::new(),
                    is_here: *root == cur,
                    is_cycle: false,
                });
                if *root == cur {
                    visited.insert(*root);
                    self.expand_children(*root, child_prefix, 1, &mut visited, &mut lines);
                }
            }
        } else {
            // Current node is not an entry: it is the single root.
            lines.push(TreeLine {
                state: cur,
                prefix: String::new(),
                is_here: true,
                is_cycle: false,
            });
            visited.insert(cur);
            self.expand_children(cur, "", 1, &mut visited, &mut lines);
        }

        lines
    }

    /// Recursively append descendant lines with `tree`-style connectors.
    fn expand_children(
        &self,
        state: i32,
        parent_prefix: &str,
        depth: usize,
        visited: &mut BTreeSet<i32>,
        out: &mut Vec<TreeLine>,
    ) {
        if depth > TREE_CHILD_DEPTH {
            return;
        }
        let children = self.build_children(state);
        let last = children.len().saturating_sub(1);
        for (i, c) in children.iter().enumerate() {
            let is_last = i == last;
            let connector = if is_last { "└── " } else { "├── " };
            let child_prefix = format!("{parent_prefix}{}", if is_last { "    " } else { "│   " });

            let is_cycle = !visited.insert(c.to);
            out.push(TreeLine {
                state: c.to,
                prefix: format!("{parent_prefix}{connector}"),
                is_here: false,
                is_cycle,
            });

            if !is_cycle {
                self.expand_children(c.to, &child_prefix, depth + 1, visited, out);
            }
        }
    }

    fn loop_(&mut self, terminal: &mut Tui) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;
            if !event::poll(std::time::Duration::from_millis(250))? {
                continue;
            }
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match self.mode {
                Mode::PickEntry { .. } => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Down | KeyCode::Char('j') => self.move_entry(1),
                    KeyCode::Up | KeyCode::Char('k') => self.move_entry(-1),
                    KeyCode::Enter => {
                        let Mode::PickEntry { sel } = &self.mode else {
                            continue;
                        };
                        let Some(i) = sel.selected() else {
                            continue;
                        };
                        let Some((key, start)) = self
                            .idx
                            .entry_list()
                            .get(i)
                            .map(|(k, s)| (k.clone(), *s))
                        else {
                            continue;
                        };
                        self.start_walk(key, start);
                    }
                    _ => {}
                },
                Mode::Walk => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Tab => self.focus = self.focus.next(),
                    KeyCode::Char('r') | KeyCode::Home => self.restart(),
                    KeyCode::Left | KeyCode::Char('h') | KeyCode::Backspace => self.go_back(),
                    KeyCode::Down | KeyCode::Char('j') => match self.focus {
                        Focus::Tree => self.move_tree(1),
                        Focus::Path => self.move_path(1),
                        Focus::Children => self.move_child(1),
                    },
                    KeyCode::Up | KeyCode::Char('k') => match self.focus {
                        Focus::Tree => self.move_tree(-1),
                        Focus::Path => self.move_path(-1),
                        Focus::Children => self.move_child(-1),
                    },
                    KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter => match self.focus {
                        Focus::Tree => {
                            let lines = self.build_tree_lines();
                            if let Some(line) =
                                lines.get(self.tree_state.selected().unwrap_or(0))
                            {
                                self.teleport(line.state);
                            }
                        }
                        Focus::Path => {
                            let i = self.path_state.selected().unwrap_or(0);
                            self.trim_path_to(i);
                        }
                        Focus::Children => {
                            let Some(cur) = self.current() else { continue };
                            let children = self.build_children(cur);
                            if let Some(c) =
                                children.get(self.child_state.selected().unwrap_or(0))
                            {
                                self.enter_child(c.to);
                            }
                        }
                    },
                    _ => {}
                },
            }
        }
    }

    fn draw(&mut self, f: &mut ratatui::Frame<'_>) {
        match &self.mode {
            Mode::PickEntry { .. } => self.draw_pick_entry(f),
            Mode::Walk => self.draw_walk(f),
        }
    }

    fn draw_pick_entry(&mut self, f: &mut ratatui::Frame<'_>) {
        let area = centered_rect(f.area(), 60, 60);

        let items: Vec<ListItem<'_>> = self
            .idx
            .entry_list()
            .iter()
            .map(|(key, start)| {
                let title = self.idx.node(*start).map(|n| n.title.as_str()).unwrap_or("?");
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            format!("  {:<16} ", key),
                            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("→ [{}] ", start),
                            Style::default().fg(Color::Gray),
                        ),
                        Span::raw(title.to_string()),
                    ]),
                    Line::from(""),
                ])
            })
            .collect();

        let block = focused_block(" Select entry point ", true);
        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol("▸ ");

        let Mode::PickEntry { sel } = &mut self.mode else {
            return;
        };
        f.render_stateful_widget(list, area, sel);

        let help = " ↑↓/jk select · Enter start · q quit ";
        let help_para = Paragraph::new(Line::from(Span::styled(
            help,
            Style::default().fg(Color::DarkGray),
        )));
        let help_area = Rect {
            x: area.x,
            y: area.bottom().saturating_sub(1),
            width: area.width,
            height: 1,
        };
        f.render_widget(help_para, help_area);
    }

    fn draw_walk(&mut self, f: &mut ratatui::Frame<'_>) {
        let outer = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(f.area());

        // ---- Left: focused tree. ----
        let tree_lines = self.build_tree_lines();
        let items: Vec<ListItem<'_>> = tree_lines
            .iter()
            .map(|l| ListItem::new(tree_line_render(self.idx, l, &self.cyclic, &self.unreachable)))
            .collect();

        let tree_title = match &self.entry_key {
            Some(k) => format!(" Tree · /{} ", k),
            None => " Tree ".to_string(),
        };
        let border = focused_block(&tree_title, self.focus == Focus::Tree);
        let list = List::new(items)
            .block(border)
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol("▸ ");
        f.render_stateful_widget(list, outer[0], &mut self.tree_state);

        // ---- Right column: Path / Current / Children. ----
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length((PATH_WINDOW + 2) as u16),
                Constraint::Min(8),
                Constraint::Min(4),
            ])
            .split(outer[1]);

        // Path.
        let path_lines = render_path(
            self.idx,
            &self.path,
            self.path_state.selected().unwrap_or(0),
            self.focus == Focus::Path,
        );
        let path_title = match &self.entry_key {
            Some(k) => format!(" Path · /{} ", k),
            None => " Path ".to_string(),
        };
        let path_block = focused_block(&path_title, self.focus == Focus::Path);
        let path_para = Paragraph::new(path_lines).block(path_block);
        f.render_widget(path_para, right[0]);

        // Current.
        let cur_lines = self
            .current()
            .and_then(|s| self.idx.node(s).map(|n| (s, n)))
            .map(|(s, n)| current_node_lines(s, n, &self.cyclic))
            .unwrap_or_else(|| {
                vec![Line::from(Span::styled(
                    "no node",
                    Style::default().fg(Color::DarkGray),
                ))]
            });
        let cur_border = focused_block(" Current ", false);
        let cur_para = Paragraph::new(cur_lines)
            .block(cur_border)
            .wrap(Wrap { trim: false });
        f.render_widget(cur_para, right[1]);

        // Children.
        let children: Vec<ChildInfo> = self
            .current()
            .map(|s| self.build_children(s))
            .unwrap_or_default();
        let child_items: Vec<ListItem<'_>> = if children.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "  (terminal node — no outgoing edges)",
                Style::default().fg(Color::DarkGray),
            )))]
        } else {
            children.iter().map(|c| ListItem::new(child_line(c))).collect()
        };
        let child_border = focused_block(" Children ", self.focus == Focus::Children);
        let child_list = List::new(child_items)
            .block(child_border)
            .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .highlight_symbol("▸ ");
        f.render_stateful_widget(child_list, right[2], &mut self.child_state);
    }
}

/// Render a single tree line (with connector prefix + node label).
fn tree_line_render<'a>(
    idx: &GraphIndex<'a>,
    line: &TreeLine,
    cyclic: &BTreeSet<i32>,
    unreachable: &BTreeSet<i32>,
) -> Line<'a> {
    let title = idx.node(line.state).map(|n| n.title.as_str()).unwrap_or("?");
    let mut spans = Vec::new();

    if !line.prefix.is_empty() {
        spans.push(Span::raw(line.prefix.clone()));
    }

    if line.is_here {
        spans.push(Span::styled(
            format!("[{}] ", line.state),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            title.to_string(),
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            "  ◄ HERE".to_string(),
            Style::default().fg(Color::DarkGray),
        ));
        if cyclic.contains(&line.state) {
            spans.push(Span::styled(" ↺", Style::default().fg(Color::Yellow)));
        }
        return Line::from(spans);
    }

    spans.push(Span::styled(
        format!("[{}] ", line.state),
        Style::default().fg(Color::Gray),
    ));
    if line.is_cycle {
        spans.push(Span::raw(title.to_string()));
        spans.push(Span::styled(" ↺", Style::default().fg(Color::Yellow)));
    } else if cyclic.contains(&line.state) {
        spans.push(Span::styled(
            format!("{} ↺", title),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
    } else if unreachable.contains(&line.state) {
        spans.push(Span::styled(
            title.to_string(),
            Style::default().fg(Color::DarkGray),
        ));
    } else {
        spans.push(Span::raw(title.to_string()));
    }
    Line::from(spans)
}

/// Render the Path pane (vertical, last PATH_WINDOW nodes, current highlighted).
fn render_path<'a>(
    idx: &GraphIndex<'a>,
    path: &[i32],
    selected: usize,
    focused: bool,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    let n = path.len();
    let start = n.saturating_sub(PATH_WINDOW);

    if start > 0 {
        lines.push(Line::from(Span::styled(
            format!("  ↑ {} more…", start),
            Style::default().fg(Color::DarkGray),
        )));
    }

    let last = n.saturating_sub(1);
    for (vi, &s) in path.iter().enumerate().skip(start) {
        let title = idx.node(s).map(|n| n.title.as_str()).unwrap_or("?");
        let is_current = vi == last;
        let is_selected = focused && vi == selected;
        let marker = if is_current {
            Span::styled(
                "  ◄ ".to_string(),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled("  │ ".to_string(), Style::default().fg(Color::DarkGray))
        };

        let id_style = if is_current {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Gray)
        };
        let title_style = if is_current {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default().add_modifier(Modifier::UNDERLINED)
        } else {
            Style::default()
        };

        let mut spans = vec![
            marker,
            Span::styled(format!("[{}] ", s), id_style),
            Span::styled(title.to_string(), title_style),
        ];
        if is_current {
            spans.push(Span::styled(
                "  current".to_string(),
                Style::default().fg(Color::DarkGray),
            ));
        }
        lines.push(Line::from(spans));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (empty)",
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines
}

fn child_line(c: &ChildInfo) -> Line<'_> {
    let labels = if c.labels.len() == 1 {
        c.labels[0].clone()
    } else {
        format!("{{{}}}", c.labels.join(", "))
    };
    let mut spans = vec![
        Span::styled(format!("[{}] ", labels), Style::default().fg(Color::Magenta)),
        Span::raw(format!("→ {} ", c.to)),
        Span::raw(c.title.clone()),
    ];
    if c.is_revisit {
        spans.push(Span::styled(
            "  ↺ visited".to_string(),
            Style::default().fg(Color::Yellow),
        ));
    }
    Line::from(spans)
}

/// Lines for the "Current" pane: identity + messages + options (no edges —
/// edges are shown as Children).
fn current_node_lines<'a>(
    state: i32,
    node: &'a Node,
    cyclic: &BTreeSet<i32>,
) -> Vec<Line<'a>> {
    let mut lines = Vec::new();

    lines.push(Line::from(vec![
        Span::styled("State ", Style::default().fg(Color::Gray)),
        Span::styled(
            state.to_string(),
            Style::default().add_modifier(Modifier::BOLD),
        ),
        Span::raw(if cyclic.contains(&state) { "  ↺ cycle" } else { "" }),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Title  ", Style::default().fg(Color::Gray)),
        Span::raw(node.title.clone()),
    ]));

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Messages",
        Style::default().fg(Color::Cyan),
    )));
    if node.messages.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (none)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for m in &node.messages {
            lines.push(Line::from(format!("  • {}", m.text)));
        }
    }

    if let Some(opts) = &node.options {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Options",
            Style::default().fg(Color::Cyan),
        )));
        for o in opts {
            lines.push(Line::from(format!("  ▸ {}", o)));
        }
    }

    lines
}

fn focused_block(title: &str, focused: bool) -> Block<'_> {
    let style = if focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(style)
}

/// Clamp a moved index to `[0, len-1]`.
fn clamp(next: i32, len: usize) -> i32 {
    if len == 0 {
        return 0;
    }
    let mut n = next;
    if n < 0 {
        n = 0;
    }
    if n as usize >= len {
        n = (len - 1) as i32;
    }
    n
}

/// A centered rectangle (percent of width / height), used for the entry picker.
fn centered_rect(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let pop = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(pop[1])[1]
}
