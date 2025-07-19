use anyhow::Result;
use std::time::Duration;
use tracing::{debug, warn};

/// Configuration for retry operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: usize,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
            multiplier: 2.0,
        }
    }
}

impl RetryConfig {
    /// Create a configuration for file I/O operations
    pub fn for_file_io() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(50),
            max_delay: Duration::from_millis(500),
            multiplier: 2.0,
        }
    }

    /// Create a configuration for network-like operations
    pub fn for_network() -> Self {
        Self {
            max_attempts: 5,
            initial_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(2),
            multiplier: 1.5,
        }
    }
}

/// Determines if an error is transient and worth retrying
pub fn is_transient_error(error: &anyhow::Error) -> bool {
    // Check for common transient I/O errors
    if let Some(io_error) = error.downcast_ref::<std::io::Error>() {
        match io_error.kind() {
            std::io::ErrorKind::TimedOut
            | std::io::ErrorKind::Interrupted
            | std::io::ErrorKind::WouldBlock
            | std::io::ErrorKind::WriteZero => true,

            // These might be transient on some file systems
            std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::AlreadyExists => true,

            // Permanent errors that shouldn't be retried
            std::io::ErrorKind::NotFound
            | std::io::ErrorKind::InvalidInput
            | std::io::ErrorKind::InvalidData
            | std::io::ErrorKind::UnexpectedEof => false,

            // For other errors, be conservative and retry
            _ => true,
        }
    } else {
        // For non-I/O errors, don't retry by default
        false
    }
}

/// Execute an operation with exponential backoff retry
pub async fn retry_with_backoff<F, T>(
    operation: F,
    config: RetryConfig,
    operation_name: &str,
) -> Result<T>
where
    F: Fn() -> Result<T>,
{
    let mut delay = config.initial_delay;
    let mut last_error = None;

    for attempt in 1..=config.max_attempts {
        match operation() {
            Ok(result) => {
                if attempt > 1 {
                    debug!(
                        operation = operation_name,
                        attempt = attempt,
                        "Operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(error) => {
                last_error = Some(error);

                if attempt < config.max_attempts {
                    // Check if the error is worth retrying
                    let should_retry = if let Some(last_err) = &last_error {
                        is_transient_error(last_err)
                    } else {
                        false
                    };

                    if !should_retry {
                        warn!(
                            operation = operation_name,
                            attempt = attempt,
                            error = %last_error.as_ref().unwrap(),
                            "Operation failed with non-transient error, not retrying"
                        );
                        break;
                    }

                    warn!(
                        operation = operation_name,
                        attempt = attempt,
                        delay_ms = delay.as_millis(),
                        error = %last_error.as_ref().unwrap(),
                        "Operation failed, retrying after delay"
                    );

                    tokio::time::sleep(delay).await;

                    // Exponential backoff with jitter
                    delay = Duration::from_millis(
                        (delay.as_millis() as f64 * config.multiplier) as u64,
                    )
                    .min(config.max_delay);

                    // Add some jitter to prevent thundering herd
                    let jitter =
                        Duration::from_millis(fastrand::u64(0..=delay.as_millis() as u64 / 10));
                    delay += jitter;
                }
            }
        }
    }

    // All attempts failed
    let final_error = last_error.unwrap();
    warn!(
        operation = operation_name,
        max_attempts = config.max_attempts,
        error = %final_error,
        "Operation failed after all retry attempts"
    );

    Err(final_error)
}

/// Synchronous version of retry with backoff (using std::thread::sleep)
pub fn retry_with_backoff_sync<F, T>(
    operation: F,
    config: RetryConfig,
    operation_name: &str,
) -> Result<T>
where
    F: Fn() -> Result<T>,
{
    let mut delay = config.initial_delay;
    let mut last_error = None;

    for attempt in 1..=config.max_attempts {
        match operation() {
            Ok(result) => {
                if attempt > 1 {
                    debug!(
                        operation = operation_name,
                        attempt = attempt,
                        "Operation succeeded after retry"
                    );
                }
                return Ok(result);
            }
            Err(error) => {
                last_error = Some(error);

                if attempt < config.max_attempts {
                    // Check if the error is worth retrying
                    let should_retry = if let Some(last_err) = &last_error {
                        is_transient_error(last_err)
                    } else {
                        false
                    };

                    if !should_retry {
                        warn!(
                            operation = operation_name,
                            attempt = attempt,
                            error = %last_error.as_ref().unwrap(),
                            "Operation failed with non-transient error, not retrying"
                        );
                        break;
                    }

                    warn!(
                        operation = operation_name,
                        attempt = attempt,
                        delay_ms = delay.as_millis(),
                        error = %last_error.as_ref().unwrap(),
                        "Operation failed, retrying after delay"
                    );

                    std::thread::sleep(delay);

                    // Exponential backoff
                    delay = Duration::from_millis(
                        (delay.as_millis() as f64 * config.multiplier) as u64,
                    )
                    .min(config.max_delay);
                }
            }
        }
    }

    // All attempts failed
    let final_error = last_error.unwrap();
    warn!(
        operation = operation_name,
        max_attempts = config.max_attempts,
        error = %final_error,
        "Operation failed after all retry attempts"
    );

    Err(final_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[test]
    fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(100));
    }

    #[test]
    fn test_retry_config_for_file_io() {
        let config = RetryConfig::for_file_io();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_delay, Duration::from_millis(50));
    }

    #[test]
    fn test_is_transient_error() {
        let transient_error = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
        let anyhow_error = anyhow::anyhow!(transient_error);
        assert!(is_transient_error(&anyhow_error));

        let permanent_error = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let anyhow_error = anyhow::anyhow!(permanent_error);
        assert!(!is_transient_error(&anyhow_error));
    }

    #[test]
    fn test_retry_eventually_succeeds() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let operation = move || {
            let count = counter_clone.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                Err(anyhow::anyhow!(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "timeout"
                )))
            } else {
                Ok("success")
            }
        };

        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            multiplier: 2.0,
        };

        let result = retry_with_backoff_sync(operation, config, "test_operation");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_retry_fails_after_max_attempts() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let operation = move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Err(anyhow::anyhow!(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                "timeout"
            )))
        };

        let config = RetryConfig {
            max_attempts: 2,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            multiplier: 2.0,
        };

        let result: anyhow::Result<&str> =
            retry_with_backoff_sync(operation, config, "test_operation");
        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn test_retry_stops_on_non_transient_error() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let operation = move || {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Err(anyhow::anyhow!(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "not found"
            )))
        };

        let config = RetryConfig {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1),
            max_delay: Duration::from_millis(10),
            multiplier: 2.0,
        };

        let result: anyhow::Result<&str> =
            retry_with_backoff_sync(operation, config, "test_operation");
        assert!(result.is_err());
        // Should only try once since it's a non-transient error
        assert_eq!(counter.load(Ordering::SeqCst), 1);
    }
}
