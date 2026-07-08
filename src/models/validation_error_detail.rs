use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ValidationErrorDetail {
    #[serde(rename = "field")]
    pub field: String,

    #[serde(rename = "code")]
    pub code: String,

    #[serde(rename = "message")]
    pub message: String,
}
