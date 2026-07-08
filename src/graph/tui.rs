//! Interactive full-screen TUI for browsing a script's state graph.
//!
//! Built on `ratatui` + `crossterm`. The left pane shows the spine layout
//! (nodes stacked vertically, cycles highlighted), the right pane shows the
//! selected node's details. Pure layout functions are split out so they could
//! be tested without a real terminal.

use std::collections::BTreeSet;
use std::io::{self, Stdout};

use crossterm::ExecutableCommand;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};

use crate::graph::{GraphIndex, GraphStyle, predicate_label};
use crate::models::Edge;

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

/// Which pane holds focus.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Focus {
    Spine,
    Detail,
}

struct App<'a> {
    idx: &'a GraphIndex<'a>,
    spine_state: ListState,
    focus: Focus,
    cyclic: BTreeSet<i32>,
    unreachable: BTreeSet<i32>,
}

impl<'a> App<'a> {
    fn new(idx: &'a GraphIndex<'_>) -> Self {
        let mut spine_state = ListState::default();
        if !idx.order().is_empty() {
            spine_state.select(Some(0));
        }
        Self {
            idx,
            spine_state,
            focus: Focus::Spine,
            cyclic: idx.cycle_states(),
            unreachable: idx.unreachable_states().into_iter().collect(),
        }
    }

    fn selected(&self) -> Option<i32> {
        self.spine_state
            .selected()
            .and_then(|i| self.idx.order().get(i).copied())
    }

    fn move_selection(&mut self, delta: i32) {
        let len = self.idx.order().len();
        if len == 0 {
            return;
        }
        let cur = self.spine_state.selected().unwrap_or(0) as i32;
        let mut next = cur + delta;
        if next < 0 {
            next = 0;
        }
        if next as usize >= len {
            next = (len - 1) as i32;
        }
        self.spine_state.select(Some(next as usize));
    }

    fn loop_(&mut self, terminal: &mut Tui) -> io::Result<()> {
        loop {
            terminal.draw(|f| self.draw(f))?;
            if !event::poll(std::time::Duration::from_millis(250))? {
                continue;
            }
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Tab => {
                        self.focus = match self.focus {
                            Focus::Spine => Focus::Detail,
                            Focus::Detail => Focus::Spine,
                        };
                    }
                    KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
                    KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
                    _ => {}
                }
            }
        }
    }

    fn draw(&mut self, f: &mut ratatui::Frame<'_>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
            .split(f.area());

        // Left: spine list.
        let items: Vec<ListItem<'_>> = self
            .idx
            .order()
            .iter()
            .map(|&s| ListItem::new(spine_line(self.idx, s, &self.cyclic, &self.unreachable)))
            .collect();

        let title = format!(
            " State graph · {} nodes · {} entries ",
            self.idx.order().len(),
            self.idx.entries().len()
        );
        let border = focused_block(&title, self.focus == Focus::Spine);
        let list = List::new(items).block(border).highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        );
        f.render_stateful_widget(list, chunks[0], &mut self.spine_state);

        // Right: detail of the selected node.
        let detail = self
            .selected()
            .and_then(|s| self.idx.node(s).map(|n| (s, n)))
            .map(|(s, n)| detail_lines(s, n, self.idx, &self.cyclic))
            .unwrap_or_else(|| vec![Line::from("no node selected")]);

        let block = focused_block(" Detail ", self.focus == Focus::Detail);
        let para = Paragraph::new(detail)
            .block(block)
            .wrap(Wrap { trim: false });
        f.render_widget(para, chunks[1]);
    }
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
    cyclic: &BTreeSet<i32>,
    unreachable: &BTreeSet<i32>,
) -> Line<'a> {
    let title = idx.node(state).map(|n| n.title.as_str()).unwrap_or("?");
    let mut spans = vec![Span::styled(
        format!("[{}] ", state),
        Style::default().fg(Color::Gray),
    )];
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

fn detail_lines<'a>(
    state: i32,
    node: &'a crate::models::Node,
    idx: &GraphIndex<'a>,
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

    // Options.
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

    // Edges.
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Edges",
        Style::default().fg(Color::Cyan),
    )));
    let edges: Vec<&Edge> = idx.edges(state);
    if edges.is_empty() {
        lines.push(Line::from(Span::styled(
            "  (none)",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for e in &edges {
            let label = predicate_label(&e.predicate);
            let is_back = idx.back_edges(state).iter().any(|b| b.to == e.to);
            let arrow = if is_back { " ↺ cycle" } else { "" };
            lines.push(Line::from(vec![
                Span::styled(
                    format!("  [{}] ", label),
                    Style::default().fg(Color::Magenta),
                ),
                Span::raw(format!("{} → {}", op_str(e), e.to)),
                Span::styled(arrow.to_string(), Style::default().fg(Color::Yellow)),
            ]));
        }
    }

    lines
}

fn op_str(e: &Edge) -> &'static str {
    match e.operation {
        crate::models::Operation::Noop => "noop",
        crate::models::Operation::Save => "save",
        crate::models::Operation::Append => "append",
    }
}
