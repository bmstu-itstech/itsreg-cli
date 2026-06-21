use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Edge {
    #[serde(rename = "predicate")]
    pub predicate: Box<models::Predicate>,

    #[serde(rename = "to")]
    pub to: i32,

    #[serde(rename = "operation")]
    pub operation: Operation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Operation {
    #[serde(rename = "noop")]
    Noop,
    #[serde(rename = "save")]
    Save,
    #[serde(rename = "append")]
    Append,
}

impl Default for Operation {
    fn default() -> Operation {
        Self::Noop
    }
}
