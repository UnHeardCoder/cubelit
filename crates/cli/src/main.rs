mod commands;
mod context;
mod resolve;
mod sink;

use clap::{Parser, Subcommand};

use context::Context;
use cubelit_core::error::CoreError;

/// Cubelit — manage game servers (same database as the desktop app).
#[derive(Parser, Debug)]
#[command(name = "cubelit", version, about, long_about = None, propagate_version = true)]
struct Cli {
    /// Override recipes directory (skips embedded recipe seeding)
    #[arg(long, global = true, value_name = "DIR")]
    recipes_dir: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Manage servers
    Server {
        #[command(subcommand)]
        sub: ServerCommand,
    },
    /// Follow container logs (Ctrl+C to stop)
    Logs {
        id: String,
        #[arg(long, default_value_t = 100_u64)]
        tail: u64,
    },
    /// Remote agent (stub)
    Agent {
        #[command(subcommand)]
        sub: AgentCommand,
    },
}

#[derive(Subcommand, Debug)]
enum ServerCommand {
    /// List all servers
    List,
    /// Create and start a new server
    Install {
        /// Recipe id (e.g. minecraft-java)
        recipe_id: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        port: Option<u16>,
    },
    Start { id: String },
    Stop { id: String },
    Restart { id: String },
    Status { id: String },
    Remove {
        id: String,
        #[arg(long)]
        keep_data: bool,
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand, Debug)]
enum AgentCommand {
    /// Start agent (not yet implemented — coming in v0.3.0)
    Start,
}

/// Initialise tracing.  Writes to `cubelit.log` in the platform data dir (same
/// file the desktop uses) so CLI runs are visible in a single log.  Falls back
/// to stderr when the log file cannot be opened (e.g. first boot, read-only FS).
fn init_tracing(data_dir: Option<&std::path::Path>) {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn,cubelit_core=info,cubelit_cli=info"));

    if let Some(dir) = data_dir {
        let log_file = dir.join("cubelit.log");
        if let Ok(file) = std::fs::OpenOptions::new().create(true).append(true).open(&log_file) {
            let _ = tracing_subscriber::fmt()
                .with_writer(std::sync::Mutex::new(file))
                .with_env_filter(filter)
                .try_init();
            return;
        }
    }

    // Fallback: log to stderr (no file available)
    let _ = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .try_init();
}

fn exit_code(err: &CoreError) -> i32 {
    match err {
        CoreError::Validation(_) => 4,
        CoreError::NotFound(_) => 5,
        CoreError::Docker(_) => 6,
        CoreError::Database(_) | CoreError::Migration(_) => 7,
        CoreError::Io(_) => 8,
    }
}

fn format_cli_error(err: &CoreError) -> String {
    match err {
        CoreError::Validation(s) => s.clone(),
        CoreError::NotFound(s) => s.clone(),
        CoreError::Docker(e) => format!("docker: {e} (is the Docker daemon running?)"),
        CoreError::Database(e) => format!("database: {e}"),
        CoreError::Migration(e) => format!("database migration: {e}"),
        CoreError::Io(e) => format!("io: {e}"),
    }
}

async fn run(cli: Cli) -> Result<(), CoreError> {
    let ctx = Context::new(cli.recipes_dir).await?;
    match cli.command {
        Command::Server { sub } => match sub {
            ServerCommand::List => commands::server::list(&ctx).await,
            ServerCommand::Install {
                recipe_id,
                name,
                port,
            } => commands::server::install(&ctx, recipe_id, name, port).await,
            ServerCommand::Start { id } => commands::server::start(&ctx, &id).await,
            ServerCommand::Stop { id } => commands::server::stop(&ctx, &id).await,
            ServerCommand::Restart { id } => commands::server::restart(&ctx, &id).await,
            ServerCommand::Status { id } => commands::server::status(&ctx, &id).await,
            ServerCommand::Remove {
                id,
                keep_data,
                yes,
            } => commands::server::remove(&ctx, &id, keep_data, yes).await,
        },
        Command::Logs { id, tail } => commands::logs::follow(&ctx, &id, tail).await,
        Command::Agent { sub } => match sub {
            AgentCommand::Start => commands::agent::start_stub(),
        },
    }
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let cli = Cli::parse();
    // Resolve data_dir early (best-effort) so tracing can write to cubelit.log.
    let data_dir_for_logging = context::resolve_data_dir().ok();
    init_tracing(data_dir_for_logging.as_deref());
    if let Err(e) = run(cli).await {
        eprintln!("error: {}", format_cli_error(&e));
        std::process::exit(exit_code(&e));
    }
    Ok(())
}

#[cfg(test)]
mod cli_parse_tests {
    use super::*;

    #[test]
    fn parse_server_list() {
        let c = Cli::try_parse_from(["cubelit", "server", "list"]).unwrap();
        assert!(matches!(
            c.command,
            Command::Server {
                sub: ServerCommand::List
            }
        ));
    }

    #[test]
    fn parse_server_install() {
        let c = Cli::try_parse_from([
            "cubelit",
            "server",
            "install",
            "minecraft-java",
            "--name",
            "srv",
            "--port",
            "25566",
        ])
        .unwrap();
        assert!(matches!(
            c.command,
            Command::Server {
                sub: ServerCommand::Install {
                    recipe_id,
                    name,
                    port: Some(25566),
                },
            } if recipe_id == "minecraft-java" && name == "srv"
        ));
    }

    #[test]
    fn parse_agent_start() {
        let c = Cli::try_parse_from(["cubelit", "agent", "start"]).unwrap();
        assert!(matches!(
            c.command,
            Command::Agent {
                sub: AgentCommand::Start,
            }
        ));
    }

    #[test]
    fn parse_logs_tail() {
        let c = Cli::try_parse_from(["cubelit", "logs", "abc", "--tail", "50"]).unwrap();
        assert!(matches!(
            c.command,
            Command::Logs { id, tail: 50 } if id == "abc"
        ));
    }

    #[test]
    fn install_requires_recipe_positional() {
        assert!(Cli::try_parse_from(["cubelit", "server", "install", "--name", "x"]).is_err());
    }

    #[test]
    fn unknown_subcommand_fails() {
        assert!(Cli::try_parse_from(["cubelit", "nope"]).is_err());
    }

    #[test]
    fn tail_rejects_non_numeric() {
        assert!(Cli::try_parse_from(["cubelit", "logs", "x", "--tail", "x"]).is_err());
    }

    #[test]
    fn parse_server_remove_yes() {
        let c = Cli::try_parse_from([
            "cubelit",
            "server",
            "remove",
            "srv",
            "--yes",
            "--keep-data",
        ])
        .unwrap();
        assert!(matches!(
            c.command,
            Command::Server {
                sub: ServerCommand::Remove {
                    id,
                    keep_data: true,
                    yes: true,
                },
            } if id == "srv"
        ));
    }

    #[test]
    fn global_recipes_dir() {
        let c = Cli::try_parse_from([
            "cubelit",
            "--recipes-dir",
            "/tmp/r",
            "server",
            "list",
        ])
        .unwrap();
        assert_eq!(
            c.recipes_dir,
            Some(std::path::PathBuf::from("/tmp/r"))
        );
    }
}
