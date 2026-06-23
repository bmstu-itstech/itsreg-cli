use reqwest::StatusCode;

use crate::api::{Api, ApiError};
use crate::models::Run;

pub async fn get_runs(api: &Api) -> Result<Vec<Run>, ApiError> {
    let uri = format!("{}/runs", api.base_url);
    let req = api
        .client
        .request(reqwest::Method::GET, &uri)
        .bearer_auth(api.bearer_access_token.clone())
        .build()?;

    let resp = api.client.execute(req).await?;
    let status = resp.status();
    let content = resp.text().await?;

    if status.is_success() {
        serde_json::from_str(content.as_str()).map_err(ApiError::from)
    } else {
        match status {
            StatusCode::UNAUTHORIZED => Err(ApiError::Unauthorized),
            StatusCode::INTERNAL_SERVER_ERROR => Err(ApiError::InternalServerError),
            _ => Err(ApiError::Unknown(content)),
        }
    }
}
