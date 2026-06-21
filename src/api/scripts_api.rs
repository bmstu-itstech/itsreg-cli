use reqwest::StatusCode;

use crate::api::{Api, ApiError};
use crate::models::Script;

pub async fn get_scripts(api: &Api) -> Result<Vec<Script>, ApiError> {
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
        serde_json::from_str(&content).map_err(ApiError::from)
    } else {
        match status {
            StatusCode::UNAUTHORIZED => Err(ApiError::Unauthorized),
            StatusCode::INTERNAL_SERVER_ERROR => Err(ApiError::InternalServerError),
            _ => Err(ApiError::Unknown(content)),
        }
    }
}

pub async fn get_script(api: &Api, id: &str) -> Result<Script, ApiError> {
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
        serde_json::from_str(&content).map_err(ApiError::from)
    } else {
        match status {
            StatusCode::UNAUTHORIZED => Err(ApiError::Unauthorized),
            StatusCode::NOT_FOUND => Err(ApiError::ScriptNotFound(id.to_owned())),
            StatusCode::INTERNAL_SERVER_ERROR => Err(ApiError::InternalServerError),
            _ => Err(ApiError::Unknown(content)),
        }
    }
}
