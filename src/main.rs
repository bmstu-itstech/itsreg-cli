use clap::CommandFactory;
mod api;
mod controller;
mod error;
mod models;
mod sources;
mod views;

use std::fmt::Debug;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Error as ClapError;
use clap::error::ErrorKind as ClapErrorKind;
use clap::{Parser, Subcommand, ValueEnum};
use console::Style;

use crate::api::Api;
use crate::controller::Controller;
use crate::error::CliError;
use crate::models::RunStatus;
use crate::sources::file_source::FileSource;
use crate::views::Viewer;
use crate::views::json_view::JsonViewer;
use crate::views::pretty_view::PrettyViewer;
use crate::views::table_view::TableViewer;

#[derive(ValueEnum, Clone, Debug, PartialEq, Eq)]
enum OutputFormat {
    #[clap(help = "Docker-like spaced table format")]
    Table,

    #[clap(help = "Human-readable pretty-printed table format")]
    Pretty,

    #[clap(help = "JSON format for machine processing")]
    Json,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum RunStatusCli {
    Starting,
    Active,
    Failed,
    Stopping,
    Stopped,
}

impl From<RunStatusCli> for RunStatus {
    fn from(value: RunStatusCli) -> Self {
        match value {
            RunStatusCli::Starting => Self::Starting,
            RunStatusCli::Active => Self::Active,
            RunStatusCli::Failed => Self::Failed,
            RunStatusCli::Stopping => Self::Stopping,
            RunStatusCli::Stopped => Self::Stopped,
        }
    }
}

#[derive(Parser)]
#[command(name = "itsreg")]
#[command(author = "Kirill Zhikharev")]
#[command(about = "CLI client for itsreg API - manage telegram bots")]
struct Cli {
    #[arg(
        short = 'u',
        long,
        env = "API_URL",
        default_value = "https://itsreg.itsbmstu.ru/api/v3",
        help = "Base API URL for itsreg service (including version)"
    )]
    api_url: String,

    #[arg(
        short = 't',
        long,
        env = "TOKEN",
        help = "JWT authentication token (can be set via TOKEN env var)"
    )]
    token: String,

    #[arg(short = 'f', long, default_value = "table", help = "Output format")]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[clap(about = "Manage bots")]
    Bots {
        #[clap(subcommand)]
        action: BotsCommands,
    },

    #[clap(about = "Manage scripts")]
    Scripts {
        #[clap(subcommand)]
        action: ScriptsCommands,
    },

    #[clap(about = "Manage bot runs")]
    Runs {
        #[clap(subcommand)]
        action: RunsCommands,
    },
}

#[derive(Subcommand)]
enum BotsCommands {
    #[clap(alias = "ls")]
    #[clap(about = "List all bots")]
    List,

    #[clap(about = "Get information about a specific bot")]
    Get { id: String },

    #[clap(about = "Create a new Telegram bot")]
    Create {
        #[arg(long, help = "ID of the script to associate with the bot")]
        script_id: String,

        #[arg(long, help = "Telegram bot token (obtain from @BotFather)")]
        bot_token: String,

        #[arg(long, help = "Description of the bot's purpose")]
        desc: String,
    },

    #[clap(about = "Update an existing bot's configuration")]
    Update {
        #[clap(help = "ID of the bot to update")]
        id: String,

        #[arg(long, help = "New script ID (optional)")]
        script_id: Option<String>,

        #[arg(long, help = "New bot token (optional)")]
        bot_token: Option<String>,

        #[arg(long, help = "New description (optional)")]
        desc: Option<String>,
    },

    #[clap(alias = "rm")]
    #[clap(about = "Remove/delete a bot")]
    Remove {
        #[clap(help = "ID of the bot to remove")]
        id: String,
    },

    #[clap(about = "Create a run for a specifiec bot and start it")]
    Start {
        #[clap(help = "ID of the bot to start")]
        id: String,
    },
}

#[derive(Subcommand)]
enum ScriptsCommands {
    #[clap(about = "Create a new script")]
    Create {
        #[arg(long, short = 'i', help = "Path to the script file")]
        input: PathBuf,
    },

    #[clap(alias = "ls")]
    #[clap(about = "List all available scripts")]
    List,

    #[clap(about = "Get detailed information about a specific script")]
    Get {
        #[clap(help = "ID of the script to retrieve")]
        id: String,
    },

    Update {
        #[clap(help = "Update an existent script")]
        id: String,

        #[clap(long, short = 'i', help = "Path to the script file")]
        input: PathBuf,
    },

    #[clap(alias = "rm")]
    #[clap(about = "Remove/delete a script")]
    Remove {
        #[clap(help = "ID of the script to remove")]
        id: String,
    },
}

#[derive(Subcommand)]
enum RunsCommands {
    #[clap(alias = "ls")]
    #[clap(about = "List all bot runs")]
    List {
        #[clap(long, help = "Filter runs by the bot ID")]
        bot_id: Option<String>,

        #[clap(long, help = "Filter runs with the status")]
        status: Option<RunStatusCli>,
    },

    #[clap(about = "Get information about a specific run")]
    Get {
        #[clap(help = "ID of the run to retrieve")]
        id: String,
    },

    #[clap(about = "Stop a specific run")]
    Stop {
        #[clap(help = "ID of the run to stop")]
        id: String,
    },
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
            } => ctrl.create_bot(script_id, bot_token, desc).await,
            BotsCommands::Update {
                id,
                script_id,
                bot_token,
                desc,
            } => ctrl.update_bot(id, script_id, bot_token, desc).await,
            BotsCommands::Remove { id } => ctrl.delete_bot(id).await,
            BotsCommands::Start { id } => ctrl.start_bot(id).await,
        },
        Commands::Scripts { action } => match action {
            ScriptsCommands::Create { input } => {
                let src = FileSource::new(&input);
                ctrl.create_script(&src).await
            }
            ScriptsCommands::List => ctrl.list_scripts().await,
            ScriptsCommands::Get { id } => ctrl.show_script(id).await,
            ScriptsCommands::Update { id, input } => {
                let src = FileSource::new(&input);
                ctrl.update_script(id, &src).await
            }
            ScriptsCommands::Remove { id } => ctrl.delete_script(id).await,
        },
        Commands::Runs { action } => match action {
            RunsCommands::List { bot_id, status } => {
                ctrl.view_runs(bot_id, status.map(Into::into)).await
            }
            RunsCommands::Get { id } => ctrl.show_run(id).await,
            RunsCommands::Stop { id } => ctrl.stop_run(id).await,
        },
    };

    match res {
        Ok(_) => ExitCode::SUCCESS,
        Err(err) => handle_error(err),
    }
}

fn handle_error(err: CliError) -> ExitCode {
    let mut cmd = Cli::command();

    let bold = Style::new().bold();
    let highlight = Style::new().yellow().bold();

    match err {
        CliError::InvalidInput(err) => {
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

        CliError::Unauthorized => {
            ClapError::raw(ClapErrorKind::InvalidValue, "token is expired or invalid")
        }

        CliError::BotNotFound(id) => ClapError::raw(
            ClapErrorKind::InvalidValue,
            format!("bot not found: {}", highlight.apply_to(id)),
        ),

        CliError::ScriptNotFound(id) => ClapError::raw(
            ClapErrorKind::InvalidValue,
            format!("script not found: {}", highlight.apply_to(id)),
        ),

        CliError::BotAlreadyRunning(id) => ClapError::raw(
            ClapErrorKind::InvalidValue,
            format!("bot already running: {}", highlight.apply_to(id)),
        ),

        CliError::RunNotFound(id) => ClapError::raw(
            ClapErrorKind::InvalidValue,
            format!("run not found: {}", highlight.apply_to(id)),
        ),

        CliError::InternalServerError => ClapError::raw(ClapErrorKind::Io, "internal server error"),

        CliError::Unknown(msg) => ClapError::raw(ClapErrorKind::InvalidValue, msg),

        CliError::IO(io_err) => ClapError::raw(ClapErrorKind::Io, io_err.to_string()),

        CliError::Reqwest(reqwest_err) => {
            ClapError::raw(ClapErrorKind::Io, reqwest_err.to_string())
        }

        CliError::Serde(reqwest_err) => ClapError::raw(ClapErrorKind::Io, reqwest_err.to_string()),
    }
    .format(&mut cmd)
    .print()
    .expect("Something went wrong");
    ExitCode::FAILURE
}
