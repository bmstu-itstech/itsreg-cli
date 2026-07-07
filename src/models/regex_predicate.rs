use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegexPredicate {
    #[serde(rename = "pattern")]
    pub pattern: String,
}
