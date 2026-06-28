use crate::views::truncate_with_dots;
use chrono::{DateTime, FixedOffset};
use prettytable::{Cell, Row, Table, format, row};
use textwrap::wrap;

use crate::models::{Bot, Run, RunStatus, Script};
use crate::views::Viewer;

const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const COLUMN_MAX_LENGTH: usize = 20;

#[derive(Default)]
pub struct PrettyViewer;

impl Viewer for PrettyViewer {
    fn view_bot(&self, bot: &Bot) {
        self.with_table(|t| {
            t.add_row(row![b => "ID", "OwnerID", "Desc", "ScriptID", "CreatedAt", "UpdatedAt"]);
            t.add_row(row![
                bot.id,
                bot.owner_id,
                textwrap::wrap(&bot.desc, COLUMN_MAX_LENGTH).join("\n"),
                bot.script_id,
                bot.created_at.format(TIMESTAMP_FORMAT),
                bot.updated_at.format(TIMESTAMP_FORMAT),
            ]);
        });
    }

    fn view_bots(&self, bots: &[Bot]) {
        self.with_table(|t| {
            t.add_row(row![b => "ID", "OwnerID", "Desc", "ScriptID", "CreatedAt", "UpdatedAt"]);
            for bot in bots {
                t.add_row(row![
                    bot.id,
                    bot.owner_id,
                    truncate_with_dots(&bot.desc, COLUMN_MAX_LENGTH),
                    bot.script_id,
                    bot.created_at.format(TIMESTAMP_FORMAT),
                    bot.updated_at.format(TIMESTAMP_FORMAT),
                ]);
            }
        });
    }

    fn view_bot_id(&self, id: &str) {
        println!("{}", id);
    }

    fn view_script(&self, script: &Script) {
        self.with_table(|t| {
            t.add_row(row![b => "ID", "Desc", "Entries", "Nodes", "CreatedAt", "UpdatedAt"]);
            t.add_row(row![
                script.id,
                textwrap::wrap(&script.desc, COLUMN_MAX_LENGTH).join("\n"),
                script.nodes.len(),
                script.entries.len(),
                script.created_at.format(TIMESTAMP_FORMAT),
                script.updated_at.format(TIMESTAMP_FORMAT),
            ]);
        });
    }

    fn view_scripts(&self, scripts: &[Script]) {
        self.with_table(|t| {
            t.add_row(row![b => "ID", "Desc", "Entries", "Nodes", "CreatedAt", "UpdatedAt"]);
            for script in scripts {
                t.add_row(row![
                    script.id,
                    truncate_with_dots(&script.desc, COLUMN_MAX_LENGTH),
                    script.nodes.len(),
                    script.entries.len(),
                    script.created_at.format(TIMESTAMP_FORMAT),
                    script.updated_at.format(TIMESTAMP_FORMAT),
                ]);
            }
        });
    }

    fn view_script_id(&self, id: &str) {
        println!("{}", id);
    }

    fn view_runs(&self, runs: &[Run]) {
        self.with_table(|t| {
            t.add_row(row![b => "ID", "BotID", "Status", "StartedAt", "StoppedAt", "Error"]);
            for run in runs {
                t.add_row(Self::run_to_row(run));
            }
        });
    }

    fn view_run(&self, run: &Run) {
        self.with_table(|t| {
            t.add_row(row![b => "ID", "BotID", "Status", "StartedAt", "StoppedAt", "Error"]);
            t.add_row(Self::run_to_row(run));
        })
    }

    fn view_run_id(&self, id: &str) {
        println!("{}", id);
    }
}

impl PrettyViewer {
    fn with_table(&self, f: impl FnOnce(&mut Table)) {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_BOX_CHARS);
        f(&mut table);
        table
            .print_tty(false)
            .map(|_| ())
            .unwrap_or_else(|e| eprintln!("Failed to print to table: {:?}", e));
    }

    fn run_to_row(run: &Run) -> Row {
        let mut cells = Vec::new();
        cells.push(Cell::new(&run.id));
        cells.push(Cell::new(&run.bot_id));
        let status_cell = Cell::new(run.status.as_str());
        let status_cell = match run.status {
            RunStatus::Active => status_cell.style_spec("Fg"),
            RunStatus::Failed => status_cell.style_spec("Fr"),
            RunStatus::Starting | RunStatus::Stopping => status_cell.style_spec("Fy"),
            RunStatus::Stopped => status_cell,
        };
        cells.push(status_cell);
        cells.push(Cell::new(&Self::format_date_or_na(run.started_at)));
        cells.push(Cell::new(&Self::format_date_or_na(run.stopped_at)));
        cells.push(Cell::new(
            &run.error_msg
                .clone()
                .map(|s| wrap(&s, 80).join("\n"))
                .unwrap_or_default(),
        ));
        Row::new(cells)
    }

    fn format_date_or_na(t: Option<DateTime<FixedOffset>>) -> String {
        t.map(|t| t.format(TIMESTAMP_FORMAT).to_string())
            .unwrap_or("N/A".into())
    }
}
