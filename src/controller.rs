use crate::api;
use crate::api::Api;
use crate::error::CliError;
use crate::graph::{self, GraphIndex, GraphStyle};
use crate::models::{CreateBotRequest, RunStatus, UpdateBotRequest};
use crate::sources::Source;
use crate::views::Viewer;

pub struct Controller {
    viewer: Box<dyn Viewer>,
    api: Api,
}

impl Controller {
    pub fn new(viewer: Box<dyn Viewer>, api: Api) -> Self {
        Self { viewer, api }
    }

    pub async fn list_bots(&self) -> Result<(), CliError> {
        api::bots_api::get_bots(&self.api)
            .await
            .map(|bots| self.viewer.view_bots(&bots))
    }

    pub async fn show_bot(&self, id: &str) -> Result<(), CliError> {
        api::bots_api::get_bot(&self.api, id)
            .await
            .map(|bot| self.viewer.view_bot(&bot))
    }

    pub async fn create_bot(
        &self,
        script_id: String,
        token: String,
        desc: String,
    ) -> Result<(), CliError> {
        let req = CreateBotRequest {
            script_id,
            token,
            desc,
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
    ) -> Result<(), CliError> {
        let req = UpdateBotRequest {
            script_id: script_id.map(|s| s.to_owned()),
            token: token.map(|s| s.to_owned()),
            desc: desc.map(|s| s.to_owned()),
        };
        api::bots_api::update_bot(&self.api, &bot_id, req)
            .await
            .map(|bot| self.viewer.view_bot(&bot))
    }

    pub async fn delete_bot(&self, bot_id: String) -> Result<(), CliError> {
        api::bots_api::delete_bot(&self.api, &bot_id)
            .await
            .map(|_| self.viewer.view_bot_id(&bot_id))
    }

    pub async fn create_script(&self, src: &dyn Source) -> Result<(), CliError> {
        let script = src.input_script().map_err(CliError::IO)?;
        api::scripts_api::create_script(&self.api, script)
            .await
            .map(|res| self.viewer.view_script_id(&res.script_id))
    }

    pub async fn list_scripts(&self) -> Result<(), CliError> {
        api::scripts_api::get_scripts(&self.api)
            .await
            .map(|scripts| self.viewer.view_scripts(&scripts))
    }

    pub async fn show_script(&self, id: String) -> Result<(), CliError> {
        api::scripts_api::get_script(&self.api, &id)
            .await
            .map(|script| self.viewer.view_script(&script))
    }

    pub async fn update_script(&self, id: String, src: &dyn Source) -> Result<(), CliError> {
        let script = src.input_script().map_err(CliError::IO)?;
        api::scripts_api::update_script(&self.api, &id, script)
            .await
            .map(|script| self.viewer.view_script(&script))
    }

    pub async fn delete_script(&self, id: String) -> Result<(), CliError> {
        api::scripts_api::delete_script(&self.api, &id)
            .await
            .map(|_| self.viewer.view_script_id(&id))
    }

    /// Render a script's state graph from a local file (no network needed).
    ///
    /// Launches the interactive TUI unless `plain` is set, in which case the
    /// chosen style is printed to stdout — handy for piping and CI.
    pub fn show_graph(
        &self,
        src: &dyn Source,
        style: GraphStyle,
        plain: bool,
    ) -> Result<(), CliError> {
        let script = src.input_script().map_err(CliError::IO)?;
        let idx = GraphIndex::new(&script);
        if plain {
            println!("{}", graph::render::render(&idx, style));
            Ok(())
        } else {
            graph::tui::run(&idx, style).map_err(CliError::IO)
        }
    }

    pub async fn start_bot(&self, bot_id: String) -> Result<(), CliError> {
        api::runs_api::create_run(&self.api, &bot_id)
            .await
            .map(|res| self.viewer.view_run_id(&res.run_id))
    }

    pub async fn view_runs(
        &self,
        bot_id: Option<String>,
        status: Option<RunStatus>,
    ) -> Result<(), CliError> {
        api::runs_api::get_runs(&self.api, bot_id.as_deref(), status)
            .await
            .map(|runs| self.viewer.view_runs(&runs))
    }

    pub async fn show_run(&self, id: String) -> Result<(), CliError> {
        api::runs_api::get_run(&self.api, &id)
            .await
            .map(|run| self.viewer.view_run(&run))
    }

    pub async fn stop_run(&self, id: String) -> Result<(), CliError> {
        api::runs_api::stop_run(&self.api, &id)
            .await
            .map(|_| self.viewer.view_run_id(&id))
    }
}
