use crate::models::{Bot, Run, Script};

pub mod json_view;
pub mod pretty_view;
pub mod table_view;

pub trait Viewer {
    fn view_bot(&self, bot: &Bot);
    fn view_bots(&self, bots: &[Bot]);
    fn view_bot_id(&self, id: &str);
    fn view_script(&self, script: &Script);
    fn view_scripts(&self, scripts: &[Script]);
    fn view_script_id(&self, id: &str);
    fn view_runs(&self, runs: &[Run]);
    fn view_run_id(&self, id: &str);
}

fn truncate_with_dots(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_string();
    }
    // Take the first N characters and append three dots
    format!("{}...", s.chars().take(max_chars).collect::<String>())
}
