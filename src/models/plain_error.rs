use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlainError {
    #[serde(rename = "message")]
    pub message: String,
}
