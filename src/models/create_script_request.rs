use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreateScriptRequest {
    #[serde(rename = "desc")]
    pub desc: String,

    #[serde(rename = "nodes")]
    pub nodes: Vec<models::Node>,

    #[serde(rename = "entries")]
    pub entries: Vec<models::Entry>,
}
