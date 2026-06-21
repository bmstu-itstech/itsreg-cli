use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Entry {
    #[serde(rename = "key")]
    pub key: String,

    #[serde(rename = "start")]
    pub start: i32,
}
