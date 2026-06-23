use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreateScriptResponse {
    #[serde(rename = "scriptID")]
    pub script_id: String,
}
