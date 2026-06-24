use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreateRunResponse {
    #[serde(rename = "runID")]
    pub run_id: String,
}
