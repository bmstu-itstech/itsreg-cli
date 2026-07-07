use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Predicate {
    #[serde(rename = "always")]
    Always(Box<models::AlwaysPredicate>),

    #[serde(rename = "exact")]
    Exact(Box<models::ExactPredicate>),

    #[serde(rename = "regex")]
    Regex(Box<models::RegexPredicate>),

    #[serde(untagged)]
    Unknown(serde_json::Value),
}

impl Default for Predicate {
    fn default() -> Self {
        Self::Always(Default::default())
    }
}
