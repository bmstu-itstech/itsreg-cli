use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Run {
    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "botID")]
    pub bot_id: String,

    #[serde(rename = "status")]
    pub status: models::RunStatus,

    #[serde(rename = "errorMsg", skip_serializing_if = "Option::is_none")]
    pub error_msg: Option<String>,

    #[serde(rename = "startedAt", skip_serializing_if = "Option::is_none")]
    pub started_at: Option<chrono::DateTime<chrono::FixedOffset>>,

    #[serde(rename = "stoppedAt", skip_serializing_if = "Option::is_none")]
    pub stopped_at: Option<chrono::DateTime<chrono::FixedOffset>>,
}
