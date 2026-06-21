use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    #[serde(rename = "state")]
    pub state: i32,
    
    #[serde(rename = "title")]
    pub title: String,
    
    #[serde(rename = "edges", skip_serializing_if = "Option::is_none")]
    pub edges: Option<Vec<models::Edge>>,
    
    #[serde(rename = "messages")]
    pub messages: Vec<models::Message>,
    
    #[serde(rename = "options", skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
}
