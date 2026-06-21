use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Script {
    #[serde(rename = "id")]
    pub id: String,
    
    #[serde(rename = "desc")]
    pub desc: String,
    
    #[serde(rename = "nodes")]
    pub nodes: Vec<models::Node>,
    
    #[serde(rename = "entries")]
    pub entries: Vec<models::Entry>,
    
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}
