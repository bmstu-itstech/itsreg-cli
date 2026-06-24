use crate::models::ValidationError;

#[derive(Debug)]
pub enum CliError {
    InvalidInput(ValidationError),
    BotNotFound(String),
    ScriptNotFound(String),
    BotAlreadyRunning(String),
    RunNotFound(String),
    Unauthorized,
    Unknown(String),
    InternalServerError,
    IO(std::io::Error),
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
}

impl From<reqwest::Error> for CliError {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        Self::IO(e)
    }
}
