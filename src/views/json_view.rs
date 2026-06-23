use crate::models::{Bot, Run, Script};
use crate::views::Viewer;

pub struct JsonViewer;

impl Viewer for JsonViewer {
    fn view_bot(&self, bot: &Bot) {
        let s = serde_json::to_string(bot).unwrap_or_else(|e| {
            eprintln!("Failed to serialize bot: {:?}", e);
            String::new()
        });
        println!("{}", s);
    }

    fn view_bots(&self, bots: &[Bot]) {
        let s = serde_json::to_string(bots).unwrap_or_else(|e| {
            eprintln!("Failed to serialize bots: {:?}", e);
            String::new()
        });
        println!("{}", s);
    }

    fn view_bot_id(&self, id: &str) {
        println!("{}", id);
    }

    fn view_script(&self, script: &Script) {
        let s = serde_json::to_string(script).unwrap_or_else(|e| {
            eprintln!("Failed to serialize script: {:?}", e);
            String::new()
        });
        println!("{}", s);
    }

    fn view_scripts(&self, scripts: &[Script]) {
        let s = serde_json::to_string(scripts).unwrap_or_else(|e| {
            eprintln!("Failed to serialize scripts: {:?}", e);
            String::new()
        });
        println!("{}", s);
    }

    fn view_runs(&self, runs: &[Run]) {
        let s = serde_json::to_string(runs).unwrap_or_else(|e| {
            eprintln!("Failed to serialize runs: {:?}", e);
            String::new()
        });
        println!("{}", s);
    }
}
