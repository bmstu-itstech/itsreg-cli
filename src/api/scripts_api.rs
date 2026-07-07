use reqwest::StatusCode;

use crate::api::Api;
use crate::error::CliError;
use crate::models::{CreateScriptRequest, CreateScriptResponse, Script, ValidationError};

pub async fn create_script(
    api: &Api,
    body: CreateScriptRequest,
) -> Result<CreateScriptResponse, CliError> {
    let uri = format!("{}/scripts", api.base_url);
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

pub async fn get_scripts(api: &Api) -> Result<Vec<Script>, CliError> {
    let uri = format!("{}/scripts", api.base_url);
    let req = api
        .client
        .request(reqwest::Method::GET, &uri)
        .bearer_auth(api.bearer_access_token.clone())
        .build()?;

    let resp = api.client.execute(req).await?;
    let status = resp.status();
    let content = resp.text().await?;

    if status.is_success() {
        serde_json::from_str(&content).map_err(CliError::from)
    } else {
        match status {
            StatusCode::UNAUTHORIZED => Err(CliError::Unauthorized),
            StatusCode::INTERNAL_SERVER_ERROR => Err(CliError::InternalServerError),
            _ => Err(CliError::Unknown(content)),
        }
    }
}

pub async fn get_script(api: &Api, id: &str) -> Result<Script, CliError> {
    let uri = format!("{}/scripts/{id}", api.base_url);
    let req = api
        .client
        .request(reqwest::Method::GET, &uri)
        .bearer_auth(api.bearer_access_token.clone())
        .build()?;

    let resp = api.client.execute(req).await?;
    let status = resp.status();
    let content = resp.text().await?;

    if status.is_success() {
        serde_json::from_str(&content).map_err(CliError::from)
    } else {
        match status {
            StatusCode::UNAUTHORIZED => Err(CliError::Unauthorized),
            StatusCode::NOT_FOUND => Err(CliError::ScriptNotFound(id.to_owned())),
            StatusCode::INTERNAL_SERVER_ERROR => Err(CliError::InternalServerError),
            _ => Err(CliError::Unknown(content)),
        }
    }
}

pub async fn update_script(
    api: &Api,
    id: &str,
    body: CreateScriptRequest,
) -> Result<Script, CliError> {
    let uri = format!("{}/scripts/{id}", api.base_url);
    let req = api
        .client
        .request(reqwest::Method::PUT, &uri)
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
            StatusCode::NOT_FOUND => Err(CliError::ScriptNotFound(id.to_owned())),
            StatusCode::INTERNAL_SERVER_ERROR => Err(CliError::InternalServerError),
            _ => Err(CliError::Unknown(content)),
        }
    }
}

pub async fn delete_script(api: &Api, id: &str) -> Result<(), CliError> {
    let uri = format!("{}/scripts/{id}", api.base_url);
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
            StatusCode::NOT_FOUND => Err(CliError::ScriptNotFound(id.to_owned())),
            StatusCode::INTERNAL_SERVER_ERROR => Err(CliError::InternalServerError),
            _ => Err(CliError::Unknown(content)),
        }
    }
}
