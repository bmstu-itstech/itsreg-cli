use reqwest::StatusCode;

use crate::api::Api;
use crate::error::CliError;
use crate::models::{CreateRunResponse, Run, RunStatus};

pub async fn create_run(api: &Api, bot_id: &str) -> Result<CreateRunResponse, CliError> {
    let uri = format!("{}/bots/{bot_id}/runs", api.base_url);
    let req = api
        .client
        .request(reqwest::Method::POST, &uri)
        .bearer_auth(api.bearer_access_token.clone())
        .build()?;

    let resp = api.client.execute(req).await?;
    let status = resp.status();
    let content = resp.text().await?;

    if status.is_success() {
        serde_json::from_str(content.as_str()).map_err(CliError::from)
    } else {
        match status {
            StatusCode::UNAUTHORIZED => Err(CliError::Unauthorized),
            StatusCode::NOT_FOUND => Err(CliError::BotNotFound(bot_id.to_string())),
            StatusCode::CONFLICT => Err(CliError::BotAlreadyRunning(bot_id.to_string())),
            StatusCode::INTERNAL_SERVER_ERROR => Err(CliError::InternalServerError),
            _ => Err(CliError::Unknown(content)),
        }
    }
}

pub async fn get_runs(
    api: &Api,
    f_bot_id: Option<&str>,
    f_status: Option<RunStatus>,
) -> Result<Vec<Run>, CliError> {
    let uri = format!("{}/runs", api.base_url);

    let mut query = Vec::new();
    if let Some(bot_id) = f_bot_id {
        query.push(("bot_id", bot_id));
    }
    if let Some(status) = f_status {
        query.push(("status", status.as_str()));
    }

    let req = api
        .client
        .request(reqwest::Method::GET, &uri)
        .query(&query)
        .bearer_auth(api.bearer_access_token.clone())
        .build()?;

    let resp = api.client.execute(req).await?;
    let status = resp.status();
    let content = resp.text().await?;

    if status.is_success() {
        serde_json::from_str(content.as_str()).map_err(CliError::from)
    } else {
        match status {
            StatusCode::UNAUTHORIZED => Err(CliError::Unauthorized),
            StatusCode::INTERNAL_SERVER_ERROR => Err(CliError::InternalServerError),
            _ => Err(CliError::Unknown(content)),
        }
    }
}

pub async fn stop_run(api: &Api, id: &str) -> Result<(), CliError> {
    let uri = format!("{}/runs/{id}/stop", api.base_url);
    let req = api
        .client
        .request(reqwest::Method::POST, &uri)
        .bearer_auth(api.bearer_access_token.clone())
        .build()?;

    let resp = api.client.execute(req).await?;
    let status = resp.status();
    let content = resp.text().await?;

    if status.is_success() {
        serde_json::from_str(content.as_str()).map_err(CliError::from)
    } else {
        match status {
            StatusCode::UNAUTHORIZED => Err(CliError::Unauthorized),
            StatusCode::NOT_FOUND => Err(CliError::RunNotFound(id.to_string())),
            StatusCode::INTERNAL_SERVER_ERROR => Err(CliError::InternalServerError),
            _ => Err(CliError::Unknown(content)),
        }
    }
}
