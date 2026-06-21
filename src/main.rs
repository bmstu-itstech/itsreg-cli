use clap::CommandFactory;
mod api;
mod models;
mod presenter;
mod views;

use std::fmt::Debug;
use std::process::ExitCode;

use clap::Error as ClapError;
use clap::error::ErrorKind as ClapErrorKind;
use clap::{Parser, Subcommand, ValueEnum};

use crate::api::{Api, ApiError};
use crate::presenter::Presenter;
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
        token: String,

        #[arg(long)]
        desc: String,
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

#[tokio::main]
async fn main() -> ExitCode {
    let cli: Cli = Cli::parse();

    let viewer: Box<dyn Viewer> = match cli.format {
        OutputFormat::Table => Box::new(TableViewer),
        OutputFormat::Json => Box::new(JsonViewer),
        OutputFormat::Pretty => Box::new(PrettyViewer::new(true)),
    };

    let api = Api::new(cli.api_url, cli.token);
    let presenter = Presenter::new(viewer, api);

    let res = match cli.command {
        Commands::Bots { action } => match action {
            BotsCommands::List => presenter.list_bots().await,
            BotsCommands::Get { id } => presenter.show_bot(&id).await,
            BotsCommands::Create {
                script_id,
                token,
                desc,
            } => presenter.create_bot(&script_id, &token, &desc).await,
        },
        Commands::Scripts { action } => match action {
            ScriptsCommands::List => presenter.list_scripts().await,
            ScriptsCommands::Get { id } => presenter.show_script(&id).await,
        },
    };

    match res {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => handle_error(err),
    }
}

fn handle_error(err: ApiError) -> ExitCode {
    let mut cmd = Cli::command();
    match err {
        ApiError::InvalidInput(err) => {
            let mut msg = String::new();
            msg.push_str(&format!("validation failed with {} errors", err.len()));
            for det in err {
                msg.push_str(&format!(
                    "\n\tin field {}: {} (code {})",
                    det.field, det.message, det.code
                ));
            }
            ClapError::raw(ClapErrorKind::InvalidValue, msg)
        }

        ApiError::Unauthorized => {
            ClapError::raw(ClapErrorKind::InvalidValue, "token is expired or invalid")
        }

        ApiError::BotNotFound(id) => ClapError::raw(
            ClapErrorKind::InvalidValue,
            format!("bot not found: {}", id),
        ),

        ApiError::ScriptNotFound(id) => ClapError::raw(
            ClapErrorKind::InvalidValue,
            format!("script not found: {}", id),
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
