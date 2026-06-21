use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreateBotResponse {
    #[serde(rename = "botID")]
    pub bot_id: String,
}
