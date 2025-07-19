use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use memoranda::cli::{DoctorCommand, HelpCommand};
use memoranda::config::Settings;
use memoranda::error::{CliError, MemorandaError};
use memoranda::logging;
use memoranda::mcp::McpServer;
use tracing::{Level, error, info, span, warn};

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
    Doctor {
        /// Show verbose output with detailed information
        #[arg(long)]
        verbose: bool,

        /// Attempt to automatically fix issues
        #[arg(long)]
        auto_fix: bool,
    },
    /// Start the MCP server
    Serve,
}

#[tokio::main]
async fn main() {
    // Initialize logging from environment variables with fallback
    if let Err(e) = logging::init_logging_from_env() {
        eprintln!("Warning: Failed to initialize logging from environment: {e}");
        eprintln!("Falling back to basic logging configuration");
        if let Err(basic_err) = logging::init_basic_logging() {
            eprintln!("Fatal: Failed to initialize basic logging: {basic_err}");
            std::process::exit(78); // EX_CONFIG
        }
    }

    let _span = span!(Level::INFO, "main").entered();
    info!(
        version = env!("CARGO_PKG_VERSION"),
        build_time = std::env::var("BUILD_TIME").unwrap_or_else(|_| "unknown".to_string()),
        git_commit = std::env::var("GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string()),
        "Starting memoranda"
    );

    let result = run_cli().await;

    // Handle errors with appropriate exit codes and user-friendly messages
    match result {
        Ok(()) => {
            info!("memoranda completed successfully");
            std::process::exit(0);
        }
        Err(e) => {
            // Log the full error chain for debugging
            error!(error = %e, "Application error occurred");

            // Extract user-friendly error message
            let user_message = extract_user_friendly_message(&e);
            eprintln!("Error: {user_message}");

            // Provide suggestions if possible
            if let Some(suggestion) = get_error_suggestion(&e) {
                eprintln!("Suggestion: {suggestion}");
            }

            // Exit with appropriate code based on error type
            let exit_code = determine_exit_code(&e);
            std::process::exit(exit_code);
        }
    }
}

/// Extract a user-friendly error message from the error chain
fn extract_user_friendly_message(error: &anyhow::Error) -> String {
    // Check if this is one of our custom error types with user-friendly messages
    if let Some(memoranda_error) = error.downcast_ref::<MemorandaError>() {
        return format_memoranda_error(memoranda_error);
    }

    // For other errors, use the main error message
    error.to_string()
}

/// Format MemorandaError for user-friendly display
fn format_memoranda_error(error: &MemorandaError) -> String {
    match error {
        MemorandaError::Config { message, .. } => {
            format!("Configuration issue: {message}")
        }
        MemorandaError::Storage { message, .. } => {
            format!("File system issue: {message}")
        }
        MemorandaError::McpServer { message, .. } => {
            format!("MCP server issue: {message}")
        }
        MemorandaError::Cli { message, .. } => {
            format!("Command error: {message}")
        }
        MemorandaError::Validation { message } => {
            format!("Invalid input: {message}")
        }
        MemorandaError::Io(io_error) => {
            format!("File operation failed: {io_error}")
        }
        MemorandaError::Json(json_error) => {
            format!("Data format error: {json_error}")
        }
    }
}

/// Provide helpful suggestions based on error type
fn get_error_suggestion(error: &anyhow::Error) -> Option<String> {
    if let Some(memoranda_error) = error.downcast_ref::<MemorandaError>() {
        match memoranda_error {
            MemorandaError::Config { .. } => {
                Some("Check your configuration file or run 'memoranda doctor' to diagnose issues".to_string())
            }
            MemorandaError::Storage { .. } => {
                Some("Check file permissions and available disk space, or run 'memoranda doctor --auto-fix'".to_string())
            }
            MemorandaError::McpServer { .. } => {
                Some("Ensure no other process is using the port, or check server logs for details".to_string())
            }
            MemorandaError::Cli { .. } => {
                Some("Use 'memoranda --help' to see available commands and options".to_string())
            }
            MemorandaError::Validation { .. } => {
                Some("Check your input parameters and try again".to_string())
            }
            MemorandaError::Io(io_error) => {
                match io_error.kind() {
                    std::io::ErrorKind::NotFound => {
                        Some("Ensure the file or directory exists and the path is correct".to_string())
                    }
                    std::io::ErrorKind::PermissionDenied => {
                        Some("Check file permissions or run with appropriate privileges".to_string())
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    } else {
        None
    }
}

/// Determine appropriate exit code based on error type
fn determine_exit_code(error: &anyhow::Error) -> i32 {
    if let Some(memoranda_error) = error.downcast_ref::<MemorandaError>() {
        match memoranda_error {
            MemorandaError::Config { .. } => 78,     // EX_CONFIG
            MemorandaError::Storage { .. } => 74,    // EX_IOERR
            MemorandaError::McpServer { .. } => 69,  // EX_UNAVAILABLE
            MemorandaError::Cli { .. } => 64,        // EX_USAGE
            MemorandaError::Validation { .. } => 65, // EX_DATAERR
            MemorandaError::Io(io_error) => match io_error.kind() {
                std::io::ErrorKind::NotFound => 2,
                std::io::ErrorKind::PermissionDenied => 77, // EX_NOPERM
                _ => 74,                                    // EX_IOERR
            },
            MemorandaError::Json(_) => 65, // EX_DATAERR
        }
    } else {
        1 // Generic error
    }
}

/// Print help for the doctor subcommand
fn print_doctor_help() {
    println!("memoranda-doctor");
    println!("Check system health and configuration");
    println!();
    println!("Usage:");
    println!("    memoranda doctor [OPTIONS]");
    println!();
    println!("Options:");
    println!("        --auto-fix    Attempt to automatically fix issues");
    println!("    -h, --help        Print help");
    println!("        --verbose     Show verbose output with detailed information");
}

/// Print help for the serve subcommand  
fn print_serve_help() {
    println!("memoranda-serve");
    println!("Start the MCP server");
    println!();
    println!("Usage:");
    println!("    memoranda serve");
    println!();
    println!("Options:");
    println!("    -h, --help    Print help");
}

async fn run_cli() -> Result<()> {
    let _span = span!(Level::INFO, "run_cli").entered();

    // Check for special cases before parsing
    let args: Vec<String> = std::env::args().collect();

    // Handle global flags and help command
    if args.len() == 2 {
        match args[1].as_str() {
            "help" => {
                let _cmd_span = span!(Level::INFO, "help_command").entered();
                info!("Showing help information via help subcommand");
                let help = HelpCommand::new();
                help.run();
                return Ok(());
            }
            "--help" | "-h" => {
                let _cmd_span = span!(Level::INFO, "help_command").entered();
                info!("Showing help information via help flag");
                let help = HelpCommand::new();
                help.run();
                return Ok(());
            }
            "--version" | "-V" => {
                let _cmd_span = span!(Level::INFO, "version_command").entered();
                info!("Showing version information");
                println!("memoranda {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            _ => {}
        }
    }

    // Handle subcommand help flags
    if args.len() == 3 && (args[2] == "--help" || args[2] == "-h") {
        match args[1].as_str() {
            "doctor" => {
                let _cmd_span = span!(Level::INFO, "doctor_help").entered();
                info!("Showing doctor command help");
                print_doctor_help();
                return Ok(());
            }
            "serve" => {
                let _cmd_span = span!(Level::INFO, "serve_help").entered();
                info!("Showing serve command help");
                print_serve_help();
                return Ok(());
            }
            _ => {}
        }
    }

    // Parse CLI arguments with better error context
    let cli = Cli::try_parse().map_err(|e| {
        // For invalid subcommands, preserve Clap's original error message
        let error_msg = e.to_string();
        if error_msg.contains("unrecognized subcommand") {
            anyhow::anyhow!("{}", error_msg)
        } else {
            CliError::invalid_argument("command line", error_msg).into()
        }
    })?;

    // Initialize settings with better error handling and context
    let _settings = Settings::new()
        .context("Failed to initialize application settings")
        .map_err(|e| {
            warn!("Settings initialization failed, using defaults");
            anyhow::anyhow!("Settings error: {}", e)
        })?;

    // Execute the requested command with proper error context
    match &cli.command {
        Some(Commands::Doctor { verbose, auto_fix }) => {
            let _cmd_span = span!(
                Level::INFO,
                "doctor_command",
                verbose = verbose,
                auto_fix = auto_fix
            )
            .entered();
            info!(
                verbose = verbose,
                auto_fix = auto_fix,
                "Running doctor command"
            );

            let doctor = DoctorCommand::with_options(*verbose, *auto_fix);
            doctor
                .run()
                .await
                .context("Doctor command execution failed")?;
        }
        Some(Commands::Serve) => {
            let _cmd_span = span!(Level::INFO, "serve_command").entered();
            info!("Starting MCP server");

            let mut server = McpServer::new("memoranda".to_string())
                .context("Failed to initialize MCP server")
                .map_err(|e| {
                    error!(error = %e, "MCP server initialization failed");
                    e
                })?;

            server
                .start()
                .await
                .context("MCP server startup failed")
                .map_err(|e| {
                    error!(error = %e, "MCP server execution failed");
                    e
                })?;
        }
        None => {
            let _cmd_span = span!(Level::INFO, "help_command").entered();
            info!("Showing help information");

            let help = HelpCommand::new();
            help.run();
        }
    }

    Ok(())
}
