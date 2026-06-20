use crate::models::Bot;

pub mod json_view;
pub mod table_view;

pub trait Viewer {
    fn view_bot(&self, bot: &Bot);
    fn view_bots(&self, bots: &[Bot]);
}
