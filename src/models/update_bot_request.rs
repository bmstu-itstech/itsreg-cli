use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct UpdateBotRequest {
    #[serde(rename = "scriptID", skip_serializing_if = "Option::is_none")]
    pub script_id: Option<String>,

    #[serde(rename = "token", skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    #[serde(rename = "desc", skip_serializing_if = "Option::is_none")]
    pub desc: Option<String>,
}
