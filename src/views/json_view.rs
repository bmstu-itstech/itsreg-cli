use crate::models::Bot;
use crate::views::Viewer;

pub struct JsonViewer;

impl Viewer for JsonViewer {
    fn view_bot(&self, bot: &Bot) {
        let s = serde_json::to_string(bot)
            .unwrap_or_else(|e| {
                eprintln!("Failed to serialize bot: {:?}", e);
                String::new()
            });
        println!("{}", s);
    }
    
    fn view_bots(&self, bots: &[Bot]) {
        let s = serde_json::to_string(bots)
            .unwrap_or_else(|e| {
                eprintln!("Failed to serialize bots: {:?}", e);
                String::new()
            });
        println!("{}", s);
    }
}
