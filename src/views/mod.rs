use crate::models::{Bot, Script};

pub mod json_view;
pub mod table_view;

pub trait Viewer {
    fn view_bot(&self, bot: &Bot);
    fn view_bots(&self, bots: &[Bot]);
    fn view_bot_id(&self, id: &str);
    fn view_script(&self, script: &Script);
    fn view_scripts(&self, scripts: &[Script]);
}
