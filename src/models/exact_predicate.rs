use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExactPredicate {
    #[serde(rename = "text")]
    pub text: String,
}
