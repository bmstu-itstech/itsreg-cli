use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreateBotRequest {
    #[serde(rename = "scriptID")]
    pub script_id: String,

    #[serde(rename = "token")]
    pub token: String,

    #[serde(rename = "desc")]
    pub desc: String,
}
