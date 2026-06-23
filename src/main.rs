use clap::CommandFactory;
mod api;
mod controller;
mod models;
mod views;

use std::fmt::Debug;
use std::process::ExitCode;

use clap::Error as ClapError;
use clap::error::ErrorKind as ClapErrorKind;
use clap::{Parser, Subcommand, ValueEnum};
use console::Style;

use crate::api::{Api, ApiError};
use crate::controller::Controller;
use crate::views::Viewer;
use crate::views::json_view::JsonViewer;
use crate::views::pretty_view::PrettyViewer;
use crate::views::table_view::TableViewer;

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
enum OutputFormat {
    Table,
    Pretty,
    Json,
}

#[derive(Parser)]
#[command(name = "itsreg")]
#[command(author = "Kirill Zhikharev")]
#[command(about = "CLI client for itsreg API")]
struct Cli {
    #[arg(
        short = 'u',
        env = "API_URL",
        default_value = "https://itsreg.itsbmstu.ru/api/v3"
    )]
    api_url: String,

    #[arg(short = 't', env = "TOKEN", help = "JWT token")]
    token: String,

    #[arg(short = 'f', default_value = "table")]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Bots {
        #[clap(subcommand)]
        action: BotsCommands,
    },
    Scripts {
        #[clap(subcommand)]
        action: ScriptsCommands,
    },
    Runs {
        #[clap(subcommand)]
        action: RunsCommands,
    },
}

#[derive(Subcommand)]
enum BotsCommands {
    #[clap(alias = "ls")]
    List,

    Get {
        id: String,
    },

    Create {
        #[arg(long)]
        script_id: String,

        #[arg(long)]
        bot_token: String,

        #[arg(long)]
        desc: String,
    },

    Update {
        id: String,

        #[arg(long)]
        script_id: Option<String>,

        #[arg(long)]
        bot_token: Option<String>,

        #[arg(long)]
        desc: Option<String>,
    },

    #[clap(alias = "rm")]
    Remove {
        id: String,
    },
}

#[derive(Subcommand)]
enum ScriptsCommands {
    #[clap(alias = "ls")]
    List,
    Get {
        id: String,
    },
}

#[derive(Subcommand)]
enum RunsCommands {
    #[clap(alias = "ls")]
    List,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli: Cli = Cli::parse();

    let viewer: Box<dyn Viewer> = match cli.format {
        OutputFormat::Table => Box::new(TableViewer),
        OutputFormat::Json => Box::new(JsonViewer),
        OutputFormat::Pretty => Box::new(PrettyViewer),
    };

    let api = Api::new(cli.api_url, cli.token);
    let ctrl = Controller::new(viewer, api);

    let res = match cli.command {
        Commands::Bots { action } => match action {
            BotsCommands::List => ctrl.list_bots().await,
            BotsCommands::Get { id } => ctrl.show_bot(&id).await,
            BotsCommands::Create {
                script_id,
                bot_token,
                desc,
            } => ctrl.create_bot(&script_id, &bot_token, &desc).await,
            BotsCommands::Update {
                id,
                script_id,
                bot_token,
                desc,
            } => ctrl.update_bot(id, script_id, bot_token, desc).await,
            BotsCommands::Remove { id } => ctrl.delete_bot(&id).await,
        },
        Commands::Scripts { action } => match action {
            ScriptsCommands::List => ctrl.list_scripts().await,
            ScriptsCommands::Get { id } => ctrl.show_script(&id).await,
        },
        Commands::Runs { action } => match action {
            RunsCommands::List => ctrl.view_runs().await,
        },
    };

    match res {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => handle_error(err),
    }
}

fn handle_error(err: ApiError) -> ExitCode {
    let mut cmd = Cli::command();

    let bold = Style::new().bold();
    let highlight = Style::new().yellow().bold();

    match err {
        ApiError::InvalidInput(err) => {
            let mut msg = String::new();
            msg.push_str(&format!("validation failed with {} errors\n", err.len()));
            for det in err {
                msg.push_str(&format!(
                    "\n\tin field {}: {} (code '{}')",
                    bold.apply_to(det.field),
                    det.message,
                    highlight.apply_to(det.code),
                ));
            }
            ClapError::raw(ClapErrorKind::InvalidValue, msg)
        }

        ApiError::Unauthorized => {
            ClapError::raw(ClapErrorKind::InvalidValue, "token is expired or invalid")
        }

        ApiError::BotNotFound(id) => ClapError::raw(
            ClapErrorKind::InvalidValue,
            format!("bot not found: {}", highlight.apply_to(id)),
        ),

        ApiError::ScriptNotFound(id) => ClapError::raw(
            ClapErrorKind::InvalidValue,
            format!("script not found: {}", highlight.apply_to(id)),
        ),

        ApiError::InternalServerError => ClapError::raw(ClapErrorKind::Io, "internal server error"),

        ApiError::Unknown(msg) => ClapError::raw(ClapErrorKind::InvalidValue, msg),

        ApiError::Reqwest(reqwest_err) => {
            ClapError::raw(ClapErrorKind::Io, reqwest_err.to_string())
        }

        ApiError::Serde(reqwest_err) => ClapError::raw(ClapErrorKind::Io, reqwest_err.to_string()),
    }
    .format(&mut cmd)
    .print()
    .expect("Something went wrong");
    ExitCode::FAILURE
}
