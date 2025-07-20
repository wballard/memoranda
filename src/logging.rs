use std::env;
use std::fs::{self, File};
use std::path::Path;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

use crate::error::{MemorandaError, Result};

/// Output format configuration
#[derive(Debug, Clone)]
pub enum OutputFormat {
    /// Plain text output with optional colors
    Plain { use_colors: bool },
    /// JSON format output
    Json,
}

/// What information to include in logs
#[derive(Debug, Clone)]
pub struct IncludeOptions {
    /// Whether to include file and line number in logs
    pub location: bool,
    /// Whether to include span information
    pub spans: bool,
}

/// Configuration for the logging system
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log level filter (e.g., "debug", "info", "warn", "error")
    pub level: String,
    /// Optional file path for log output
    pub file_path: Option<String>,
    /// Output format configuration
    pub output_format: OutputFormat,
    /// What information to include in logs
    pub include: IncludeOptions,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file_path: Some("./memoranda/mcp.log".to_string()),
            output_format: OutputFormat::Plain { use_colors: true },
            include: IncludeOptions {
                location: false,
                spans: true,
            },
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
        let json_format = if let Ok(json) = env::var("MEMORANDA_LOG_JSON") {
            json.parse().unwrap_or(false)
        } else {
            false
        };

        // Set color usage from environment
        let use_colors = if let Ok(colors) = env::var("MEMORANDA_LOG_COLORS") {
            colors.parse().unwrap_or(true)
        } else {
            true
        };

        // Set output format based on JSON and colors settings
        config.output_format = if json_format {
            OutputFormat::Json
        } else {
            OutputFormat::Plain { use_colors }
        };

        // Set file output from environment
        if let Ok(file_path) = env::var("MEMORANDA_LOG_FILE") {
            if !file_path.is_empty() {
                config.file_path = Some(file_path);
            }
        }

        // Set location inclusion from environment
        if let Ok(location) = env::var("MEMORANDA_LOG_LOCATION") {
            config.include.location = location.parse().unwrap_or(false);
        }

        // Set span inclusion from environment
        if let Ok(spans) = env::var("MEMORANDA_LOG_SPANS") {
            config.include.spans = spans.parse().unwrap_or(true);
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
    let layer = match &config.output_format {
        OutputFormat::Json => {
            // JSON format layer
            fmt::layer()
                .json()
                .with_target(true)
                .with_file(config.include.location)
                .with_line_number(config.include.location)
                .with_span_events(if config.include.spans {
                    FmtSpan::NEW | FmtSpan::CLOSE
                } else {
                    FmtSpan::NONE
                })
                .boxed()
        }
        OutputFormat::Plain { use_colors } => {
            // Compact format layer
            fmt::layer()
                .compact()
                .with_target(true)
                .with_file(config.include.location)
                .with_line_number(config.include.location)
                .with_ansi(*use_colors)
                .with_span_events(if config.include.spans {
                    FmtSpan::NEW | FmtSpan::CLOSE
                } else {
                    FmtSpan::NONE
                })
                .boxed()
        }
    };

    // Initialize the subscriber
    if let Some(file_path) = &config.file_path {
        // Ensure the parent directory exists
        if let Some(parent) = Path::new(file_path).parent() {
            fs::create_dir_all(parent).map_err(|e| {
                MemorandaError::config_with_source(
                    format!("Failed to create log directory: {}", parent.display()),
                    e,
                )
            })?;
        }

        // Log to file
        let file = File::create(file_path).map_err(|e| {
            MemorandaError::config_with_source(format!("Failed to create log file: {file_path}"), e)
        })?;

        let file_layer = fmt::layer()
            .with_writer(file)
            .with_target(true)
            .with_file(config.include.location)
            .with_line_number(config.include.location)
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
        output_format = ?config.output_format,
        file_path = ?config.file_path,
        include_location = config.include.location,
        include_spans = config.include.spans,
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
    use std::collections::HashMap;
    use std::env;

    /// Safe environment variable management for tests
    struct TestEnvironment {
        original_values: HashMap<String, Option<String>>,
    }

    impl TestEnvironment {
        fn new() -> Self {
            Self {
                original_values: HashMap::new(),
            }
        }

        fn set_var(&mut self, key: &str, value: &str) {
            // Store the original value before changing it
            if !self.original_values.contains_key(key) {
                self.original_values
                    .insert(key.to_string(), env::var(key).ok());
            }
            // SAFETY: This unsafe block is safe because:
            // 1. We store the original value before modification to ensure restoration
            // 2. The Drop trait guarantees cleanup even if the test panics
            // 3. Environment variable modification is atomic at the OS level
            // 4. Test isolation is maintained by the encapsulation pattern
            unsafe {
                env::set_var(key, value);
            }
        }

        fn remove_var(&mut self, key: &str) {
            // Store the original value before removing it
            if !self.original_values.contains_key(key) {
                self.original_values
                    .insert(key.to_string(), env::var(key).ok());
            }
            // SAFETY: This unsafe block is safe because:
            // 1. We store the original value before removal to enable restoration
            // 2. The Drop trait guarantees cleanup even if the test panics
            // 3. Environment variable removal is atomic at the OS level
            // 4. Test isolation is maintained by the encapsulation pattern
            unsafe {
                env::remove_var(key);
            }
        }
    }

    impl Drop for TestEnvironment {
        fn drop(&mut self) {
            // Restore all environment variables to their original state
            // SAFETY: This cleanup operation is safe because:
            // 1. We are only restoring values that were previously captured
            // 2. The original_values HashMap contains validated environment state
            // 3. This restoration maintains the exact pre-test environment
            // 4. Failure to restore would break test isolation guarantees
            // 5. Environment variable operations are atomic at the OS level
            unsafe {
                for (key, original_value) in &self.original_values {
                    match original_value {
                        Some(value) => env::set_var(key, value),
                        None => env::remove_var(key),
                    }
                }
            }
        }
    }

    #[test]
    fn test_default_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.file_path, Some("./memoranda/mcp.log".to_string()));
        assert!(!config.include.location);
        assert!(config.include.spans);
        match config.output_format {
            OutputFormat::Plain { use_colors } => assert!(use_colors),
            OutputFormat::Json => panic!("Expected plain format"),
        }
    }

    #[test]
    fn test_config_from_env() {
        let mut test_env = TestEnvironment::new();

        // Clear any interfering environment variables first
        test_env.remove_var("RUST_LOG");
        test_env.remove_var("MEMORANDA_LOG_LEVEL");

        // Set environment variables safely
        test_env.set_var("MEMORANDA_LOG_LEVEL", "debug");
        test_env.set_var("MEMORANDA_LOG_JSON", "true");
        test_env.set_var("MEMORANDA_LOG_LOCATION", "true");
        test_env.set_var("MEMORANDA_LOG_SPANS", "false");
        test_env.set_var("MEMORANDA_LOG_COLORS", "false");

        let config = LoggingConfig::from_env().unwrap();
        assert_eq!(config.level, "debug");
        assert!(config.include.location);
        assert!(!config.include.spans);
        match config.output_format {
            OutputFormat::Json => {} // Expected
            OutputFormat::Plain { .. } => panic!("Expected JSON format"),
        }

        // Environment variables are automatically restored when test_env is dropped
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
        let mut test_env = TestEnvironment::new();

        // Test that RUST_LOG is used as fallback
        test_env.set_var("RUST_LOG", "warn");
        test_env.remove_var("MEMORANDA_LOG_LEVEL");

        let config = LoggingConfig::from_env().unwrap();
        assert_eq!(config.level, "warn");

        // Environment variables are automatically restored when test_env is dropped
    }

    #[test]
    fn test_log_file_env_override() {
        let mut test_env = TestEnvironment::new();

        // Clear any interfering environment variables
        test_env.remove_var("RUST_LOG");
        test_env.remove_var("MEMORANDA_LOG_LEVEL");

        // Test that MEMORANDA_LOG_FILE overrides the default
        test_env.set_var("MEMORANDA_LOG_FILE", "/tmp/custom.log");

        let config = LoggingConfig::from_env().unwrap();
        assert_eq!(config.file_path, Some("/tmp/custom.log".to_string()));

        // Environment variables are automatically restored when test_env is dropped
    }

    #[test]
    fn test_default_log_file_preserved_when_no_override() {
        let mut test_env = TestEnvironment::new();

        // Clear any interfering environment variables
        test_env.remove_var("RUST_LOG");
        test_env.remove_var("MEMORANDA_LOG_LEVEL");
        test_env.remove_var("MEMORANDA_LOG_FILE");

        let config = LoggingConfig::from_env().unwrap();
        assert_eq!(config.file_path, Some("./memoranda/mcp.log".to_string()));

        // Environment variables are automatically restored when test_env is dropped
    }
}
