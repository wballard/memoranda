use anyhow::Result;
use clap::{Parser, Subcommand};
use memoranda::cli::{DoctorCommand, HelpCommand};
use memoranda::config::Settings;
use memoranda::mcp::McpServer;
use tracing::info;

#[derive(Parser)]
#[command(name = "memoranda")]
#[command(about = "A note-taking MCP server for coding agents")]
#[command(author, version, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check system health and configuration
    Doctor,
    /// Start the MCP server
    Serve,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting memoranda");

    let result = run_cli().await;

    // Handle errors with appropriate exit codes
    match result {
        Ok(()) => {
            info!("memoranda completed successfully");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("Error: {e}");

            // Exit with appropriate code based on error type
            let exit_code = match e.downcast_ref::<std::io::Error>() {
                Some(io_error) => match io_error.kind() {
                    std::io::ErrorKind::NotFound => 2,
                    std::io::ErrorKind::PermissionDenied => 77,
                    _ => 1,
                },
                None => 1,
            };

            std::process::exit(exit_code);
        }
    }
}

fn wrap_command_error(operation: &str, error: anyhow::Error) -> anyhow::Error {
    anyhow::anyhow!("{operation} failed: {error}")
}

async fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    // Initialize settings with better error handling
    let _settings =
        Settings::new().map_err(|e| anyhow::anyhow!("Failed to initialize settings: {e}"))?;

    match &cli.command {
        Some(Commands::Doctor) => {
            let doctor = DoctorCommand::new();
            doctor
                .run()
                .await
                .map_err(|e| wrap_command_error("Doctor command", e))?;
        }
        Some(Commands::Serve) => {
            let server = McpServer::new("memoranda".to_string());
            server
                .start()
                .await
                .map_err(|e| wrap_command_error("MCP server", e))?;
        }
        None => {
            let help = HelpCommand::new();
            help.run()
                .map_err(|e| wrap_command_error("Help command", e))?;
        }
    }

    Ok(())
}
