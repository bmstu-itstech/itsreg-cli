use crate::models::ValidationError;

pub mod bots_api;
pub mod scripts_api;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

#[derive(Debug, Clone)]
pub struct Api {
    client: reqwest::Client,
    base_url: String,
    bearer_access_token: String,
}

impl Api {
    pub fn new(base_url: String, bearer_access_token: String) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()
            .unwrap(); // should not fail if APP_USER_AGENT is valid
        Self {
            client,
            base_url,
            bearer_access_token,
        }
    }
}

#[derive(Debug)]
pub enum ApiError {
    InvalidInput(ValidationError),
    BotNotFound(String),
    ScriptNotFound(String),
    Unauthorized,
    Unknown(String),
    InternalServerError,
    Reqwest(reqwest::Error),
    Serde(serde_json::Error),
}

impl From<reqwest::Error> for ApiError {
    fn from(e: reqwest::Error) -> Self {
        Self::Reqwest(e)
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        Self::Serde(e)
    }
}
