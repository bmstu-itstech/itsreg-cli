use reqwest::StatusCode;

use crate::api::Api;
use crate::error::CliError;
use crate::models::{Bot, CreateBotRequest, CreateBotResponse, UpdateBotRequest, ValidationError};

pub async fn get_bot(api: &Api, id: &str) -> Result<Bot, CliError> {
    let uri = format!("{}/bots/{id}", api.base_url);
    let req = api
        .client
        .request(reqwest::Method::GET, &uri)
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
            StatusCode::NOT_FOUND => Err(CliError::BotNotFound(id.to_owned())),
            StatusCode::INTERNAL_SERVER_ERROR => Err(CliError::InternalServerError),
            _ => Err(CliError::Unknown(content)),
        }
    }
}

pub async fn get_bots(api: &Api) -> Result<Vec<Bot>, CliError> {
    let uri = format!("{}/bots", api.base_url);
    let req = api
        .client
        .request(reqwest::Method::GET, &uri)
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

pub async fn create_bot(api: &Api, body: CreateBotRequest) -> Result<CreateBotResponse, CliError> {
    let uri = format!("{}/bots", api.base_url);
    let req = api
        .client
        .request(reqwest::Method::POST, &uri)
        .bearer_auth(api.bearer_access_token.clone())
        .json(&body)
        .build()?;

    let resp = api.client.execute(req).await?;
    let status = resp.status();
    let content = resp.text().await?;

    if status.is_success() {
        serde_json::from_str(content.as_str()).map_err(CliError::from)
    } else {
        match status {
            StatusCode::UNAUTHORIZED => Err(CliError::Unauthorized),
            StatusCode::BAD_REQUEST => {
                let parsed: ValidationError =
                    serde_json::from_str(&content).map_err(CliError::from)?;
                Err(CliError::InvalidInput(parsed))
            }
            StatusCode::INTERNAL_SERVER_ERROR => Err(CliError::InternalServerError),
            _ => Err(CliError::Unknown(content)),
        }
    }
}

pub async fn update_bot(api: &Api, id: &str, body: UpdateBotRequest) -> Result<Bot, CliError> {
    let uri = format!("{}/bots/{id}", api.base_url);
    let req = api
        .client
        .request(reqwest::Method::PATCH, &uri)
        .bearer_auth(api.bearer_access_token.clone())
        .json(&body)
        .build()?;

    let resp = api.client.execute(req).await?;
    let status = resp.status();
    let content = resp.text().await?;

    if status.is_success() {
        serde_json::from_str(content.as_str()).map_err(CliError::from)
    } else {
        match status {
            StatusCode::UNAUTHORIZED => Err(CliError::Unauthorized),
            StatusCode::NOT_FOUND => Err(CliError::BotNotFound(id.to_owned())),
            StatusCode::BAD_REQUEST => {
                let parsed: ValidationError =
                    serde_json::from_str(&content).map_err(CliError::from)?;
                Err(CliError::InvalidInput(parsed))
            }
            StatusCode::INTERNAL_SERVER_ERROR => Err(CliError::InternalServerError),
            _ => Err(CliError::Unknown(content)),
        }
    }
}

pub async fn delete_bot(api: &Api, id: &str) -> Result<(), CliError> {
    let uri = format!("{}/bots/{id}", api.base_url);
    let req = api
        .client
        .request(reqwest::Method::DELETE, &uri)
        .bearer_auth(api.bearer_access_token.clone())
        .build()?;

    let resp = api.client.execute(req).await?;
    let status = resp.status();

    if status.is_success() {
        Ok(())
    } else {
        let content = resp.text().await?;
        match status {
            StatusCode::UNAUTHORIZED => Err(CliError::Unauthorized),
            StatusCode::NOT_FOUND => Err(CliError::BotNotFound(id.to_owned())),
            StatusCode::INTERNAL_SERVER_ERROR => Err(CliError::InternalServerError),
            _ => Err(CliError::Unknown(content)),
        }
    }
}
