use anyhow::Result;
use clap::{Parser, Subcommand};
use memoranda::cli::{DoctorCommand, HelpCommand};
use memoranda::config::Settings;
use memoranda::mcp::McpServer;
use tracing::info;
use tracing_subscriber;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check system health and configuration
    Doctor,
    /// Start the MCP server
    Server {
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting memoranda");
    
    let cli = Cli::parse();
    let _settings = Settings::new();
    
    match &cli.command {
        Some(Commands::Doctor) => {
            let doctor = DoctorCommand::new();
            doctor.run().await?;
        }
        Some(Commands::Server { port }) => {
            let server = McpServer::new(format!("memoranda-server:{}", port));
            server.start().await?;
        }
        None => {
            let help = HelpCommand::new();
            help.run()?;
        }
    }
    
    Ok(())
}
