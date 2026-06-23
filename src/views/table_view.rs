use prettytable::{Table, format, row};

use crate::models::{Bot, Run, Script};
use crate::views::Viewer;
use crate::views::truncate_with_dots;

const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const COLUMN_MAX_LENGTH: usize = 40;

pub struct TableViewer;

impl Viewer for TableViewer {
    fn view_bot(&self, bot: &Bot) {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_CLEAN);
        table.add_row(row![b =>
            "ID",
            "OWNER_ID",
            "DESC",
            "SCRIPT_ID",
            "CREATED_AT",
            "UPDATED_AT"
        ]);
        table.add_row(row![
            bot.id,
            bot.owner_id,
            truncate_with_dots(&bot.desc, COLUMN_MAX_LENGTH),
            bot.script_id,
            bot.created_at.format(TIMESTAMP_FORMAT),
            bot.updated_at.format(TIMESTAMP_FORMAT),
        ]);
        table
            .print_tty(false)
            .map(|_| ())
            .unwrap_or_else(|e| eprintln!("Failed to print to table: {:?}", e));
    }
    fn view_bots(&self, bots: &[Bot]) {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_CLEAN);
        table.add_row(row![b =>
            "ID",
            "OWNER_ID",
            "DESC",
            "SCRIPT_ID",
            "CREATED_AT",
            "UPDATED_AT"
        ]);
        for bot in bots {
            table.add_row(row![
                bot.id,
                bot.owner_id,
                truncate_with_dots(&bot.desc, COLUMN_MAX_LENGTH),
                bot.script_id,
                bot.created_at.format(TIMESTAMP_FORMAT),
                bot.updated_at.format(TIMESTAMP_FORMAT),
            ]);
        }
        table
            .print_tty(false)
            .map(|_| ())
            .unwrap_or_else(|e| eprintln!("Failed to print to table: {:?}", e));
    }

    fn view_bot_id(&self, id: &str) {
        println!("{}", id);
    }

    fn view_script(&self, script: &Script) {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
        table.add_row(row![b =>
            "ID",
            "DESC",
            "ENTRIES",
            "NODES",
            "CREATED_AT",
            "UPDATED_AT"
        ]);
        table.add_row(row![
            script.id,
            truncate_with_dots(&script.desc, COLUMN_MAX_LENGTH),
            script.nodes.len(),
            script.entries.len(),
            script.created_at.format(TIMESTAMP_FORMAT),
            script.updated_at.format(TIMESTAMP_FORMAT),
        ]);
        table
            .print_tty(false)
            .map(|_| ())
            .unwrap_or_else(|e| eprintln!("Failed to print to table: {:?}", e));
    }

    fn view_scripts(&self, scripts: &[Script]) {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_CLEAN);
        table.add_row(row![b =>
            "ID",
            "DESC",
            "ENTRIES",
            "NODES",
            "CREATED_AT",
            "UPDATED_AT"
        ]);
        for script in scripts {
            table.add_row(row![
                script.id,
                truncate_with_dots(&script.desc, COLUMN_MAX_LENGTH),
                script.nodes.len(),
                script.entries.len(),
                script.created_at.format(TIMESTAMP_FORMAT),
                script.updated_at.format(TIMESTAMP_FORMAT),
            ]);
        }
        table
            .print_tty(false)
            .map(|_| ())
            .unwrap_or_else(|e| eprintln!("Failed to print to table: {:?}", e));
    }

    fn view_script_id(&self, id: &str) {
        println!("{}", id);
    }

    fn view_runs(&self, runs: &[Run]) {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_CLEAN);
        table.add_row(row![b => "ID", "BOT_ID", "STATUS", "STARTED_AT", "STOPPED_AT", "ERROR"]);
        for run in runs {
            table.add_row(row![
                run.id,
                run.bot_id,
                run.status,
                run.started_at
                    .map(|t| t.format(TIMESTAMP_FORMAT).to_string())
                    .unwrap_or("N/A".into()),
                run.stopped_at
                    .map(|t| t.format(TIMESTAMP_FORMAT).to_string())
                    .unwrap_or("N/A".into()),
                run.error_msg.clone().unwrap_or_default()
            ]);
        }
        table
            .print_tty(false)
            .map(|_| ())
            .unwrap_or_else(|e| eprintln!("Failed to print to table: {:?}", e));
    }
}
