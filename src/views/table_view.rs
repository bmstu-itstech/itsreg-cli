use prettytable::{Table, row, format};

use crate::models::Bot;
use crate::views::Viewer;

const COLUMN_MAX_LENGTH: usize = 20;

#[derive(Default)]
pub struct TableViewer {
    pretty: bool
}

impl TableViewer {
    pub fn new(pretty: bool) -> Self {
        TableViewer { pretty }
    }
}

impl Viewer for TableViewer {
    fn view_bot(&self, bot: &Bot) {
        let mut table = Table::new();
        if self.pretty {
            table.set_format(*format::consts::FORMAT_BOX_CHARS);
        }
        table.add_row(row!["ID", "OwnerID", "Desc", "ScriptID", "CreatedAt", "UpdatedAt"]);
        table.add_row(
            row![
                bot.id, 
                bot.owner_id, 
                textwrap::wrap(&bot.desc, COLUMN_MAX_LENGTH).join("\n"),
                bot.script_id, 
                bot.created_at.format("%Y-%m-%d %H:%M:%S"), 
                bot.updated_at.format("%Y-%m-%d %H:%M:%S"), 
            ]
        );
        table.print_tty(false)
            .map(|_| ())
            .unwrap_or_else(|e| eprintln!("Failed to print to table: {:?}", e));
    }
    
    fn view_bots(&self, bots: &[Bot]) {
        let mut table = Table::new();
        if self.pretty {
            table.set_format(*format::consts::FORMAT_BOX_CHARS);
        }
        table.add_row(row![b => "ID", "OwnerID", "Desc", "ScriptID", "CreatedAt", "UpdatedAt"]);
        
        for bot in bots {
            
            table.add_row(
                row![
                    bot.id, 
                    bot.owner_id, 
                    truncate_with_dots(&bot.desc, COLUMN_MAX_LENGTH),
                    bot.script_id, 
                    bot.created_at.format("%Y-%m-%d %H:%M:%S"), 
                    bot.updated_at.format("%Y-%m-%d %H:%M:%S"), 
                ]
            );
        }
        table.print_tty(false)
            .map(|_| ())
            .unwrap_or_else(|e| eprintln!("Failed to print to table: {:?}", e));
    }
}

fn truncate_with_dots(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    // Take the first N characters and append three dots
    format!("{}...", s.chars().take(max_chars).collect::<String>())
}
