use std::env;
use std::fs::File;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
    Layer,
};

use crate::error::{MemorandaError, Result};

/// Configuration for the logging system
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log level filter (e.g., "debug", "info", "warn", "error")
    pub level: String,
    /// Whether to use JSON output format
    pub json_format: bool,
    /// Optional file path for log output
    pub file_path: Option<String>,
    /// Whether to include file and line number in logs
    pub include_location: bool,
    /// Whether to include span information
    pub include_spans: bool,
    /// Whether to use colored output (only applies to non-JSON format)
    pub use_colors: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            json_format: false,
            file_path: None,
            include_location: false,
            include_spans: true,
            use_colors: true,
        }
    }
}

impl LoggingConfig {
    /// Create a new logging configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config = Self::default();

        // Set log level from environment
        if let Ok(level) = env::var("MEMORANDA_LOG_LEVEL") {
            config.level = level.to_lowercase();
        } else if let Ok(level) = env::var("RUST_LOG") {
            config.level = level.to_lowercase();
        }

        // Set JSON format from environment
        if let Ok(json) = env::var("MEMORANDA_LOG_JSON") {
            config.json_format = json.parse().unwrap_or(false);
        }

        // Set file output from environment
        if let Ok(file_path) = env::var("MEMORANDA_LOG_FILE") {
            if !file_path.is_empty() {
                config.file_path = Some(file_path);
            }
        }

        // Set location inclusion from environment
        if let Ok(location) = env::var("MEMORANDA_LOG_LOCATION") {
            config.include_location = location.parse().unwrap_or(false);
        }

        // Set span inclusion from environment
        if let Ok(spans) = env::var("MEMORANDA_LOG_SPANS") {
            config.include_spans = spans.parse().unwrap_or(true);
        }

        // Set color usage from environment
        if let Ok(colors) = env::var("MEMORANDA_LOG_COLORS") {
            config.use_colors = colors.parse().unwrap_or(true);
        }

        // Validate the configuration
        config.validate()?;
        Ok(config)
    }

    /// Validate the logging configuration
    fn validate(&self) -> Result<()> {
        // Validate log level
        match self.level.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {}
            _ => {
                return Err(MemorandaError::validation(format!(
                    "Invalid log level: '{}'. Must be one of: trace, debug, info, warn, error",
                    self.level
                )));
            }
        }

        Ok(())
    }
}

/// Initialize the logging system with the provided configuration
pub fn init_logging(config: &LoggingConfig) -> Result<()> {
    // Create environment filter
    let env_filter = create_env_filter(&config.level)?;

    // Create the base layer
    let layer = if config.json_format {
        // JSON format layer
        fmt::layer()
            .json()
            .with_target(true)
            .with_file(config.include_location)
            .with_line_number(config.include_location)
            .with_span_events(if config.include_spans {
                FmtSpan::NEW | FmtSpan::CLOSE
            } else {
                FmtSpan::NONE
            })
            .boxed()
    } else {
        // Compact format layer
        fmt::layer()
            .compact()
            .with_target(true)
            .with_file(config.include_location)
            .with_line_number(config.include_location)
            .with_ansi(config.use_colors)
            .with_span_events(if config.include_spans {
                FmtSpan::NEW | FmtSpan::CLOSE
            } else {
                FmtSpan::NONE
            })
            .boxed()
    };

    // Initialize the subscriber
    if let Some(file_path) = &config.file_path {
        // Log to file
        let file = File::create(file_path).map_err(|e| {
            MemorandaError::config_with_source(
                format!("Failed to create log file: {file_path}"),
                e,
            )
        })?;
        
        let file_layer = fmt::layer()
            .with_writer(file)
            .with_target(true)
            .with_file(config.include_location)
            .with_line_number(config.include_location)
            .with_ansi(false); // No colors for file output
        
        tracing_subscriber::registry()
            .with(env_filter)
            .with(file_layer)
            .init();
    } else {
        // Log to stderr
        tracing_subscriber::registry()
            .with(env_filter)
            .with(layer)
            .init();
    }

    tracing::info!(
        level = %config.level,
        json_format = config.json_format,
        file_path = ?config.file_path,
        include_location = config.include_location,
        include_spans = config.include_spans,
        use_colors = config.use_colors,
        "Logging initialized successfully"
    );

    Ok(())
}

/// Create environment filter for log levels
fn create_env_filter(level: &str) -> Result<EnvFilter> {
    let filter_str = format!("memoranda={level}");
    EnvFilter::try_new(&filter_str).map_err(|e| {
        MemorandaError::config_with_source(
            format!("Failed to create log filter with level: {level}"),
            e,
        )
    })
}

/// Initialize basic logging with sensible defaults
pub fn init_basic_logging() -> Result<()> {
    let config = LoggingConfig::default();
    init_logging(&config)
}

/// Initialize logging from environment variables
pub fn init_logging_from_env() -> Result<()> {
    let config = LoggingConfig::from_env()?;
    init_logging(&config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert!(!config.json_format);
        assert!(config.file_path.is_none());
        assert!(!config.include_location);
        assert!(config.include_spans);
        assert!(config.use_colors);
    }

    #[test]
    fn test_config_from_env() {
        // Set environment variables
        unsafe {
            env::set_var("MEMORANDA_LOG_LEVEL", "debug");
            env::set_var("MEMORANDA_LOG_JSON", "true");
            env::set_var("MEMORANDA_LOG_LOCATION", "true");
            env::set_var("MEMORANDA_LOG_SPANS", "false");
            env::set_var("MEMORANDA_LOG_COLORS", "false");
        }

        let config = LoggingConfig::from_env().unwrap();
        assert_eq!(config.level, "debug");
        assert!(config.json_format);
        assert!(config.include_location);
        assert!(!config.include_spans);
        assert!(!config.use_colors);

        // Clean up
        unsafe {
            env::remove_var("MEMORANDA_LOG_LEVEL");
            env::remove_var("MEMORANDA_LOG_JSON");
            env::remove_var("MEMORANDA_LOG_LOCATION");
            env::remove_var("MEMORANDA_LOG_SPANS");
            env::remove_var("MEMORANDA_LOG_COLORS");
        }
    }

    #[test]
    fn test_config_validation() {
        // Valid level
        let mut config = LoggingConfig { 
            level: "debug".to_string(), 
            ..Default::default() 
        };
        assert!(config.validate().is_ok());
        
        // Invalid level
        config.level = "invalid".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_env_filter_creation() {
        // Test that filter creation succeeds for valid levels
        assert!(create_env_filter("debug").is_ok());
        assert!(create_env_filter("info").is_ok());
        assert!(create_env_filter("warn").is_ok());
        assert!(create_env_filter("error").is_ok());
        assert!(create_env_filter("trace").is_ok());
    }

    #[test]
    fn test_rust_log_fallback() {
        // Test that RUST_LOG is used as fallback
        unsafe {
            env::set_var("RUST_LOG", "warn");
            env::remove_var("MEMORANDA_LOG_LEVEL");
        }
        
        let config = LoggingConfig::from_env().unwrap();
        assert_eq!(config.level, "warn");
        
        // Clean up
        unsafe {
            env::remove_var("RUST_LOG");
        }
    }
}