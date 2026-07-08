//! Interactive full-screen TUI for walking a script's state graph.
//!
//! Built on `ratatui` + `crossterm`. Instead of a flat browse, the TUI walks the
//! graph like a user following the bot: an entry-point picker leads into a
//! step-by-step traversal where each step shows the current node's details, its
//! outgoing children (pickable with Enter), and a vertical trail of how we got
//! here. The left pane keeps a read-only overview of all blocks with a `►`
//! marker tracking the current node.

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
const PATH_WINDOW: usize = 3;

/// Which screen the app is on.
#[derive(Debug)]
enum Mode {
    /// Entry-point picker shown at startup and after `r`/restart.
    PickEntry {
        sel: ListState,
    },
    /// Step-by-step graph walk.
    Walk,
}

/// Which pane holds focus during a [`Mode::Walk`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Focus {
    /// Left overview list (scroll only; does not move the walk).
    Overview,
    /// Right children list (drives the walk).
    Children,
}

/// A single reachable target from the current node, with the predicates that
/// lead to it grouped together.
#[derive(Debug)]
struct ChildInfo {
    to: i32,
    title: String,
    labels: Vec<String>,
    /// Target is already on the current walk path → returning to it (`↺`).
    is_revisit: bool,
}

struct App<'a> {
    idx: &'a GraphIndex<'a>,
    mode: Mode,
    /// Walk history; `path.last()` is the current node.
    path: Vec<i32>,
    /// Key of the entry the current walk started from (for the Path title).
    entry_key: Option<String>,
    child_state: ListState,
    overview_state: ListState,
    focus: Focus,
    cyclic: BTreeSet<i32>,
    unreachable: BTreeSet<i32>,
}

impl<'a> App<'a> {
    fn new(idx: &'a GraphIndex<'_>) -> Self {
        let mut overview_state = ListState::default();
        if !idx.order().is_empty() {
            overview_state.select(Some(0));
        }
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
            overview_state,
            focus: Focus::Children,
            cyclic: idx.cycle_states(),
            unreachable: idx.unreachable_states().into_iter().collect(),
        }
    }

    /// Current node of the walk, if any.
    fn current(&self) -> Option<i32> {
        self.path.last().copied()
    }

    /// Children (unique edge targets) of the current node.
    fn build_children(&self, state: i32) -> Vec<ChildInfo> {
        // Group edges by target, preserving first-seen order.
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
        self.focus = Focus::Children;
        self.sync_overview_to_current();
        self.mode = Mode::Walk;
    }

    /// Step into the selected child.
    fn enter_child(&mut self, to: i32) {
        self.path.push(to);
        self.child_state.select(Some(0));
        self.sync_overview_to_current();
    }

    /// Step back one node, never below the entry root.
    fn go_back(&mut self) {
        if self.path.len() <= 1 {
            return;
        }
        self.path.pop();
        self.child_state.select(Some(0));
        self.sync_overview_to_current();
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

    /// Keep the overview list scrolled so the current node stays visible.
    fn sync_overview_to_current(&mut self) {
        let Some(cur) = self.current() else {
            return;
        };
        if let Some(pos) = self.idx.order().iter().position(|&s| s == cur) {
            self.overview_state.select(Some(pos));
        }
    }

    fn move_child(&mut self, delta: i32, len: usize) {
        if len == 0 {
            return;
        }
        let cur = self.child_state.selected().unwrap_or(0) as i32;
        let mut next = cur + delta;
        if next < 0 {
            next = 0;
        }
        if next as usize >= len {
            next = (len - 1) as i32;
        }
        self.child_state.select(Some(next as usize));
    }

    fn move_overview(&mut self, delta: i32) {
        let len = self.idx.order().len();
        if len == 0 {
            return;
        }
        let cur = self.overview_state.selected().unwrap_or(0) as i32;
        let mut next = cur + delta;
        if next < 0 {
            next = 0;
        }
        if next as usize >= len {
            next = (len - 1) as i32;
        }
        self.overview_state.select(Some(next as usize));
    }

    fn move_entry(&mut self, delta: i32) {
        let len = self.idx.entry_list().len();
        if len == 0 {
            return;
        }
        let Mode::PickEntry { sel } = &mut self.mode else {
            return;
        };
        let cur = sel.selected().unwrap_or(0) as i32;
        let mut next = cur + delta;
        if next < 0 {
            next = 0;
        }
        if next as usize >= len {
            next = (len - 1) as i32;
        }
        sel.select(Some(next as usize));
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
                    KeyCode::Tab => {
                        self.focus = match self.focus {
                            Focus::Overview => Focus::Children,
                            Focus::Children => Focus::Overview,
                        };
                    }
                    KeyCode::Char('r') | KeyCode::Home => {
                        self.restart();
                    }
                    KeyCode::Left | KeyCode::Char('h') | KeyCode::Backspace => self.go_back(),
                    KeyCode::Down | KeyCode::Char('j') => match self.focus {
                        Focus::Children => {
                            let len = self
                                .current()
                                .map(|s| self.build_children(s).len())
                                .unwrap_or(0);
                            self.move_child(1, len);
                        }
                        Focus::Overview => self.move_overview(1),
                    },
                    KeyCode::Up | KeyCode::Char('k') => match self.focus {
                        Focus::Children => {
                            let len = self
                                .current()
                                .map(|s| self.build_children(s).len())
                                .unwrap_or(0);
                            self.move_child(-1, len);
                        }
                        Focus::Overview => self.move_overview(-1),
                    },
                    KeyCode::Right | KeyCode::Char('l') | KeyCode::Enter
                        if self.focus == Focus::Children =>
                    {
                        let children = self
                            .current()
                            .map(|s| self.build_children(s))
                            .unwrap_or_default();
                        let i = self.child_state.selected().unwrap_or(0);
                        if let Some(c) = children.get(i) {
                            self.enter_child(c.to);
                        }
                    }
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
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
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
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(f.area());

        // ---- Left: read-only overview with a ► marker on the current node. ----
        let cur = self.current();
        let items: Vec<ListItem<'_>> = self
            .idx
            .order()
            .iter()
            .map(|&s| {
                ListItem::new(spine_line(
                    self.idx,
                    s,
                    cur == Some(s),
                    &self.cyclic,
                    &self.unreachable,
                ))
            })
            .collect();

        let title = format!(
            " All blocks · {} ",
            self.idx.order().len(),
        );
        let border = focused_block(&title, self.focus == Focus::Overview);
        let list = List::new(items)
            .block(border)
            .highlight_style(Style::default().bg(Color::Black));
        f.render_stateful_widget(list, outer[0], &mut self.overview_state);

        // ---- Right column: Path / Current / Children. ----
        let right = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length((PATH_WINDOW + 2) as u16), // window + "..." + border
                Constraint::Min(8),
                Constraint::Min(4),
            ])
            .split(outer[1]);

        // Path.
        let path_lines = path_lines(self.idx, &self.path);
        let path_title = match &self.entry_key {
            Some(k) => format!(" Path · /{} ", k),
            None => " Path ".to_string(),
        };
        let path_para = Paragraph::new(path_lines).block(Block::default().borders(Borders::ALL).title(path_title));
        f.render_widget(path_para, right[0]);

        // Current.
        let cur_lines = self
            .current()
            .and_then(|s| self.idx.node(s).map(|n| (s, n)))
            .map(|(s, n)| current_node_lines(s, n, &self.cyclic))
            .unwrap_or_else(|| vec![Line::from(Span::styled(
                "no node",
                Style::default().fg(Color::DarkGray),
            ))]);
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
            children
                .iter()
                .map(|c| ListItem::new(child_line(c)))
                .collect()
        };
        let child_border = focused_block(" Children ", self.focus == Focus::Children);
        let child_list = List::new(child_items)
            .block(child_border)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▸ ");
        f.render_stateful_widget(child_list, right[2], &mut self.child_state);
    }
}

fn child_line(c: &ChildInfo) -> Line<'_> {
    let labels = if c.labels.len() == 1 {
        c.labels[0].clone()
    } else {
        format!("{{{}}}", c.labels.join(", "))
    };
    let mut spans = vec![
        Span::styled(
            format!("[{}] ", labels),
            Style::default().fg(Color::Magenta),
        ),
        Span::raw(format!("→ {} ", c.to)),
        Span::styled(c.title.clone(), Style::default()),
    ];
    if c.is_revisit {
        spans.push(Span::styled(
            "  ↺ visited".to_string(),
            Style::default().fg(Color::Yellow),
        ));
    }
    Line::from(spans)
}

fn path_lines<'a>(idx: &GraphIndex<'a>, path: &[i32]) -> Vec<Line<'a>> {
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
    for (i, &s) in path.iter().skip(start).enumerate() {
        let title = idx.node(s).map(|n| n.title.as_str()).unwrap_or("?");
        let is_current = start + i == last;
        if is_current {
            lines.push(Line::from(vec![
                Span::styled(
                    "  ◄ [".to_string(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    s.to_string(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "] ".to_string(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    title.to_string(),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "  current".to_string(),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        } else {
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  │ [{}] ", s),
                    Style::default().fg(Color::Gray),
                ),
                Span::raw(title.to_string()),
            ]));
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (empty)",
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines
}

/// Lines for the "Current" pane: identity + messages + options, but NOT edges
/// (edges are shown as Children).
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
        Span::raw(if cyclic.contains(&state) {
            "  ↺ cycle"
        } else {
            ""
        }),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Title  ", Style::default().fg(Color::Gray)),
        Span::raw(node.title.clone()),
    ]));

    // Messages.
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

    // Options (the node's "buttons").
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

fn spine_line<'a>(
    idx: &GraphIndex<'a>,
    state: i32,
    is_current: bool,
    cyclic: &BTreeSet<i32>,
    unreachable: &BTreeSet<i32>,
) -> Line<'a> {
    let title = idx.node(state).map(|n| n.title.as_str()).unwrap_or("?");
    let mut spans = vec![Span::styled(
        format!("[{}] ", state),
        Style::default().fg(Color::Gray),
    )];
    if is_current {
        spans.push(Span::styled(
            "► ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if cyclic.contains(&state) {
        spans.push(Span::styled(
            format!("{} ↺", title),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
    } else if unreachable.contains(&state) {
        spans.push(Span::styled(
            title.to_string(),
            Style::default().fg(Color::DarkGray),
        ));
    } else {
        spans.push(Span::raw(title.to_string()));
    }
    Line::from(spans)
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
