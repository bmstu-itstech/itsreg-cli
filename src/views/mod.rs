use std::borrow::Cow::{self, Owned};

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
    fn view_run(&self, run: &Run);
    fn view_runs(&self, runs: &[Run]);
    fn view_run_id(&self, id: &str);
}

fn truncate_with_dots<'a>(s: &'a str, max_chars: usize) -> Cow<'a, str> {
    let len = s.chars().count();
    if len <= max_chars {
        return Cow::Borrowed(s);
    }

    if len < max_chars {
        return Cow::Borrowed(s);
    }

    // Result can't have length more than `max_chars`
    if max_chars == 0 {
        return Cow::Owned("".into());
    }
    if max_chars < 3 {
        return Cow::Owned(".".repeat(max_chars));
    }

    let truncated = format!("{}...", s.chars().take(max_chars).collect::<String>());
    Owned(truncated)
}
