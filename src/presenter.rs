use crate::api;
use crate::api::{Api, ApiError};
use crate::views::Viewer;

pub struct Presenter {
    viewer: Box<dyn Viewer>,
    api: Api,
}

impl Presenter {
    pub fn new(viewer: Box<dyn Viewer>, api: Api) -> Self {
        Self { viewer, api }
    }
    
    pub async fn list_bots(&self) -> Result<(), ApiError> {
        api::bots_api::get_bots(&self.api)
            .await
            .map(|bots| self.viewer.view_bots(&bots))
    }
    
    pub async fn get_bot(&self, id: &str) -> Result<(), ApiError> {
        api::bots_api::get_bot(&self.api, id)
            .await
            .map(|bot| self.viewer.view_bot(&bot))
    }
}
