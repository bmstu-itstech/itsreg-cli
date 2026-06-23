use serde::{Deserialize, Serialize};

#[derive(
    Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize,
)]
pub enum RunStatus {
    #[serde(rename = "starting")]
    #[default]
    Starting,

    #[serde(rename = "active")]
    Active,

    #[serde(rename = "failed")]
    Failed,

    #[serde(rename = "stopping")]
    Stopping,

    #[serde(rename = "stopped")]
    Stopped,
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Starting => write!(f, "starting"),
            Self::Active => write!(f, "active"),
            Self::Failed => write!(f, "failed"),
            Self::Stopping => write!(f, "stopping"),
            Self::Stopped => write!(f, "stopped"),
        }
    }
}
