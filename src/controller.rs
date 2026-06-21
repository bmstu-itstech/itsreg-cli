use crate::api;
use crate::api::{Api, ApiError};
use crate::models::{CreateBotRequest, UpdateBotRequest};
use crate::views::Viewer;

pub struct Controller {
    viewer: Box<dyn Viewer>,
    api: Api,
}

impl Controller {
    pub fn new(viewer: Box<dyn Viewer>, api: Api) -> Self {
        Self { viewer, api }
    }

    pub async fn list_bots(&self) -> Result<(), ApiError> {
        api::bots_api::get_bots(&self.api)
            .await
            .map(|bots| self.viewer.view_bots(&bots))
    }

    pub async fn show_bot(&self, id: &str) -> Result<(), ApiError> {
        api::bots_api::get_bot(&self.api, id)
            .await
            .map(|bot| self.viewer.view_bot(&bot))
    }

    pub async fn create_bot(
        &self,
        script_id: &str,
        token: &str,
        desc: &str,
    ) -> Result<(), ApiError> {
        let req = CreateBotRequest {
            script_id: script_id.to_owned(),
            token: token.to_owned(),
            desc: desc.to_owned(),
        };
        api::bots_api::create_bot(&self.api, req)
            .await
            .map(|res| self.viewer.view_bot_id(&res.bot_id))
    }

    pub async fn update_bot(
        &self,
        bot_id: String,
        script_id: Option<String>,
        token: Option<String>,
        desc: Option<String>,
    ) -> Result<(), ApiError> {
        let req = UpdateBotRequest {
            script_id: script_id.map(|s| s.to_owned()),
            token: token.map(|s| s.to_owned()),
            desc: desc.map(|s| s.to_owned()),
        };
        api::bots_api::update_bot(&self.api, &bot_id, req)
            .await
            .map(|bot| self.viewer.view_bot(&bot))
    }

    pub async fn delete_bot(&self, bot_id: &str) -> Result<(), ApiError> {
        api::bots_api::delete_bot(&self.api, bot_id)
            .await
            .map(|_| self.viewer.view_bot_id(bot_id))
    }

    pub async fn list_scripts(&self) -> Result<(), ApiError> {
        api::scripts_api::get_scripts(&self.api)
            .await
            .map(|scripts| self.viewer.view_scripts(&scripts))
    }

    pub async fn show_script(&self, id: &str) -> Result<(), ApiError> {
        api::scripts_api::get_script(&self.api, id)
            .await
            .map(|script| self.viewer.view_script(&script))
    }
}
