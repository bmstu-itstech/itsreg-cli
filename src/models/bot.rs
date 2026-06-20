use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Bot {
    #[serde(rename = "id")]
    pub id: String,
    
    #[serde(rename = "ownerID")]
    pub owner_id: i64,
    
    #[serde(rename = "scriptID")]
    pub script_id: String,
    
    #[serde(rename = "desc")]
    pub desc: String,
    
    #[serde(rename = "createdAt")]
    pub created_at: chrono::DateTime<chrono::FixedOffset>,
    
    #[serde(rename = "updatedAt")]
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}
