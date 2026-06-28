use chrono::{DateTime, FixedOffset};
use prettytable::{Table, format, row};

use crate::models::{Bot, Run, Script};
use crate::views::Viewer;
use crate::views::truncate_with_dots;

const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const COLUMN_MAX_LENGTH: usize = 40;

pub struct TableViewer;

impl Viewer for TableViewer {
    fn view_bot(&self, bot: &Bot) {
        self.with_table(|t| {
            t.add_row(row![b =>
                "ID",
                "OWNER_ID",
                "DESC",
                "SCRIPT_ID",
                "CREATED_AT",
                "UPDATED_AT"
            ]);
            t.add_row(row![
                bot.id,
                bot.owner_id,
                truncate_with_dots(&bot.desc, COLUMN_MAX_LENGTH),
                bot.script_id,
                bot.created_at.format(TIMESTAMP_FORMAT),
                bot.updated_at.format(TIMESTAMP_FORMAT),
            ]);
        });
    }

    fn view_bots(&self, bots: &[Bot]) {
        self.with_table(|t| {
            t.add_row(row![b =>
                "ID",
                "OWNER_ID",
                "DESC",
                "SCRIPT_ID",
                "CREATED_AT",
                "UPDATED_AT"
            ]);
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
            t.add_row(row![b =>
                "ID",
                "DESC",
                "ENTRIES",
                "NODES",
                "CREATED_AT",
                "UPDATED_AT"
            ]);
            t.add_row(row![
                script.id,
                truncate_with_dots(&script.desc, COLUMN_MAX_LENGTH),
                script.nodes.len(),
                script.entries.len(),
                script.created_at.format(TIMESTAMP_FORMAT),
                script.updated_at.format(TIMESTAMP_FORMAT),
            ]);
        });
    }

    fn view_scripts(&self, scripts: &[Script]) {
        self.with_table(|t| {
            t.add_row(row![b =>
                "ID",
                "DESC",
                "ENTRIES",
                "NODES",
                "CREATED_AT",
                "UPDATED_AT"
            ]);
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

    fn view_run(&self, run: &Run) {
        self.with_table(|t| {
            t.add_row(row![b => "ID", "BOT_ID", "STATUS", "STARTED_AT", "STOPPED_AT", "ERROR"]);
            t.add_row(row![
                run.id,
                run.bot_id,
                run.status,
                Self::format_date_or_na(run.started_at),
                Self::format_date_or_na(run.stopped_at),
                run.error_msg.clone().unwrap_or_default()
            ]);
        });
    }

    fn view_runs(&self, runs: &[Run]) {
        self.with_table(|t| {
            t.add_row(row![b => "ID", "BOT_ID", "STATUS", "STARTED_AT", "STOPPED_AT", "ERROR"]);
            for run in runs {
                t.add_row(row![
                    run.id,
                    run.bot_id,
                    run.status,
                    Self::format_date_or_na(run.started_at),
                    Self::format_date_or_na(run.stopped_at),
                    run.error_msg.clone().unwrap_or_default()
                ]);
            }
        });
    }

    fn view_run_id(&self, id: &str) {
        println!("{}", id);
    }
}

impl TableViewer {
    fn with_table(&self, f: impl FnOnce(&mut Table)) {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_CLEAN);
        f(&mut table);
        table
            .print_tty(false)
            .map(|_| ())
            .unwrap_or_else(|e| eprintln!("Failed to print to t: {:?}", e));
    }

    fn format_date_or_na(t: Option<DateTime<FixedOffset>>) -> String {
        t.map(|t| t.format(TIMESTAMP_FORMAT).to_string())
            .unwrap_or("N/A".into())
    }
}
