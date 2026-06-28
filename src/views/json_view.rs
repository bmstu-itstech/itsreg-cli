use serde::Serialize;

use crate::models::{Bot, Run, Script};
use crate::views::Viewer;

pub struct JsonViewer;

impl Viewer for JsonViewer {
    fn view_bot(&self, bot: &Bot) {
        Self::print_or_error(bot);
    }

    fn view_bots(&self, bots: &[Bot]) {
        Self::print_or_error(bots);
    }

    fn view_bot_id(&self, id: &str) {
        println!("{}", id);
    }

    fn view_script(&self, script: &Script) {
        Self::print_or_error(script);
    }

    fn view_scripts(&self, scripts: &[Script]) {
        Self::print_or_error(scripts);
    }

    fn view_script_id(&self, id: &str) {
        println!("{}", id);
    }

    fn view_run(&self, run: &Run) {
        Self::print_or_error(run);
    }

    fn view_runs(&self, runs: &[Run]) {
        Self::print_or_error(runs);
    }

    fn view_run_id(&self, id: &str) {
        println!("{}", id);
    }
}

impl JsonViewer {
    fn print_or_error<T>(value: &T)
    where
        T: ?Sized + Serialize,
    {
        let stdout = std::io::stdout().lock();
        serde_json::to_writer(stdout, value).unwrap_or_else(|e| eprintln!("{:?}", e));
    }
}
