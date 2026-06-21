use crate::views::truncate_with_dots;
use prettytable::{Table, format, row};

use crate::models::{Bot, Script};
use crate::views::Viewer;

const TIMESTAMP_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const COLUMN_MAX_LENGTH: usize = 20;

#[derive(Default)]
pub struct PrettyViewer {
    styled: bool,
}

impl PrettyViewer {
    pub fn new(styled: bool) -> Self {
        PrettyViewer { styled }
    }
}

impl Viewer for PrettyViewer {
    fn view_bot(&self, bot: &Bot) {
        let mut table = Table::new();
        if self.styled {
            table.set_format(*format::consts::FORMAT_BOX_CHARS);
        }
        table.add_row(row![
            "ID",
            "OwnerID",
            "Desc",
            "ScriptID",
            "CreatedAt",
            "UpdatedAt"
        ]);
        table.add_row(row![
            bot.id,
            bot.owner_id,
            textwrap::wrap(&bot.desc, COLUMN_MAX_LENGTH).join("\n"),
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
        if self.styled {
            table.set_format(*format::consts::FORMAT_BOX_CHARS);
        }
        table.add_row(row![b => "ID", "OwnerID", "Desc", "ScriptID", "CreatedAt", "UpdatedAt"]);

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
        if self.styled {
            table.set_format(*format::consts::FORMAT_BOX_CHARS);
        }
        table.add_row(row![b => "ID", "Desc", "Entries", "Nodes", "CreatedAt", "UpdatedAt"]);
        table.add_row(row![
            script.id,
            textwrap::wrap(&script.desc, COLUMN_MAX_LENGTH).join("\n"),
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
        if self.styled {
            table.set_format(*format::consts::FORMAT_BOX_CHARS);
        }
        table.add_row(row![b => "ID", "Desc", "Entries", "Nodes", "CreatedAt", "UpdatedAt"]);
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
}
