use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlainError {
    #[serde(rename = "message")]
    pub message: String,
}
