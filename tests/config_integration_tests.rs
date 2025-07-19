use fake::Fake;
use memoranda::config::Settings;
use memoranda::error::MemorandaError;
use std::fs;
use std::path::PathBuf;
use tempfile::{NamedTempFile, TempDir};

/// Comprehensive configuration system tests with advanced validation scenarios

#[test]
fn test_settings_with_extreme_values() {
    // Test with maximum values
    let settings = Settings {
        data_dir: PathBuf::from("/very/long/path/that/goes/deep/into/filesystem/structure"),
        log_level: "trace".to_string(),
        mcp_server_port: 65535, // Maximum valid port
        minimum_rust_version: "1.80.0".to_string(),
        max_memo_file_size: u64::MAX,
    };
    assert!(settings.validate().is_ok());

    // Test with minimum values
    let settings = Settings {
        data_dir: PathBuf::from("."),
        log_level: "error".to_string(),
        mcp_server_port: 1024, // Minimum valid port
        minimum_rust_version: "1.0.0".to_string(),
        max_memo_file_size: 1,
    };
    assert!(settings.validate().is_ok());
}

#[test]
fn test_settings_validation_edge_cases() {
    // Test port boundary values
    let settings = Settings {
        data_dir: PathBuf::from("./data"),
        log_level: "info".to_string(),
        mcp_server_port: 1023, // Just below minimum
        minimum_rust_version: "1.70.0".to_string(),
        max_memo_file_size: 1_000_000,
    };
    assert!(settings.validate().is_err());

    // Test with port exactly at boundary
    let settings = Settings {
        data_dir: PathBuf::from("./data"),
        log_level: "info".to_string(),
        mcp_server_port: 1024, // Exactly at minimum
        minimum_rust_version: "1.70.0".to_string(),
        max_memo_file_size: 1_000_000,
    };
    assert!(settings.validate().is_ok());
}

#[test]
fn test_settings_rust_version_validation() {
    // Test various valid version formats
    let valid_versions = vec!["1.70.0", "1.80.1", "2.0.0"];

    for version in valid_versions {
        let settings = Settings {
            data_dir: PathBuf::from("./data"),
            log_level: "info".to_string(),
            mcp_server_port: 8080,
            minimum_rust_version: version.to_string(),
            max_memo_file_size: 1_000_000,
        };
        assert!(
            settings.validate().is_ok(),
            "Version should be valid: {version}"
        );
    }

    // Test various invalid version formats
    let invalid_versions = vec![
        "1.70",
        "1",
        "1.70.0.1",
        "1.70.0-",
        "1.70.0+",
        "v1.70.0",
        "1.70.0-beta",
        "1.70.0-alpha",
        "1.70.0-beta.1",
        "1.70.0-alpha.1",
        "1.70.0-rc.1",
        "1.70.0+build.1",
        "latest",
        "stable",
        "1.70.0-beta.1.2",
        "1.70.0++build",
    ];

    for version in invalid_versions {
        let settings = Settings {
            data_dir: PathBuf::from("./data"),
            log_level: "info".to_string(),
            mcp_server_port: 8080,
            minimum_rust_version: version.to_string(),
            max_memo_file_size: 1_000_000,
        };
        assert!(
            settings.validate().is_err(),
            "Version should be invalid: {version}"
        );
    }
}

#[test]
fn test_settings_log_level_validation() {
    // Test valid log levels
    let valid_levels = vec![
        "error", "warn", "info", "debug", "trace", "ERROR", "WARN", "INFO", "DEBUG", "TRACE",
    ];

    for level in valid_levels {
        let settings = Settings {
            data_dir: PathBuf::from("./data"),
            log_level: level.to_string(),
            mcp_server_port: 8080,
            minimum_rust_version: "1.70.0".to_string(),
            max_memo_file_size: 1_000_000,
        };
        assert!(
            settings.validate().is_ok(),
            "Log level should be valid: {level}"
        );
    }

    // Test empty log level
    let settings = Settings {
        data_dir: PathBuf::from("./data"),
        log_level: "".to_string(),
        mcp_server_port: 8080,
        minimum_rust_version: "1.70.0".to_string(),
        max_memo_file_size: 1_000_000,
    };
    assert!(settings.validate().is_err());

    // Test whitespace-only log level
    let settings = Settings {
        data_dir: PathBuf::from("./data"),
        log_level: "   ".to_string(),
        mcp_server_port: 8080,
        minimum_rust_version: "1.70.0".to_string(),
        max_memo_file_size: 1_000_000,
    };
    // Note: Current implementation doesn't trim whitespace, so this passes validation
    // This test documents the current behavior
    assert!(settings.validate().is_ok());
}

#[test]
fn test_settings_file_operations_with_unicode() {
    let temp_file = NamedTempFile::new().unwrap();
    let settings = Settings {
        data_dir: PathBuf::from("./数据"),
        log_level: "info".to_string(),
        mcp_server_port: 8080,
        minimum_rust_version: "1.70.0".to_string(),
        max_memo_file_size: 1_000_000,
    };

    // Save and load settings with unicode paths
    let path = temp_file.path().to_path_buf();
    assert!(settings.save_to_file(&path).is_ok());

    let loaded_settings = Settings::load_from_file(&path).unwrap();
    assert_eq!(settings.data_dir, loaded_settings.data_dir);
    assert_eq!(settings.log_level, loaded_settings.log_level);
}

#[test]
fn test_settings_file_operations_with_nested_directory() {
    let temp_dir = TempDir::new().unwrap();
    let nested_path = temp_dir
        .path()
        .join("config")
        .join("nested")
        .join("settings.json");

    let settings = Settings::new().unwrap();

    // Save to nested path (should create directories)
    assert!(settings.save_to_file(&nested_path).is_ok());
    assert!(nested_path.exists());

    // Load from nested path
    let loaded_settings = Settings::load_from_file(&nested_path).unwrap();
    assert_eq!(settings.data_dir, loaded_settings.data_dir);
}

#[test]
fn test_settings_file_operations_error_handling() {
    use memoranda::MemorandaError;
    let settings = Settings::new().unwrap();

    // Test saving to a path with invalid characters (null bytes)
    let invalid_path = PathBuf::from("/root/\0invalid/cannot_write_here.json");
    let result = settings.save_to_file(&invalid_path);
    assert!(
        result.is_err(),
        "Should fail to save to invalid path with null bytes"
    );
    match result.unwrap_err() {
        MemorandaError::Io(_) => {} // Expected IO error
        other => panic!("Expected IO error, got: {other:?}"),
    }

    // Test saving to a path that tries to create a file inside a non-directory file
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    let impossible_path = temp_file
        .path()
        .join("cannot_create_dir_here")
        .join("file.json");
    let result = settings.save_to_file(&impossible_path);
    assert!(
        result.is_err(),
        "Should fail to save to path inside a regular file"
    );
    match result.unwrap_err() {
        MemorandaError::Io(io_err) => {
            // Verify it's a meaningful IO error
            assert!(
                !io_err.to_string().is_empty(),
                "IO error should have descriptive message"
            );
        }
        other => panic!("Expected IO error, got: {other:?}"),
    }

    // Test loading from non-existent file (should return Ok with defaults)
    let non_existent_path = PathBuf::from("/definitely/does/not/exist/settings.json");
    let result = Settings::load_from_file(&non_existent_path);
    assert!(
        result.is_ok(),
        "Should return default settings for non-existent file"
    );
    let loaded_settings = result.unwrap();
    let default_settings = Settings::default();
    assert_eq!(loaded_settings.data_dir, default_settings.data_dir);
    assert_eq!(loaded_settings.log_level, default_settings.log_level);

    // Test loading from a directory instead of a file
    let temp_dir = tempfile::TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_path_buf();
    let result = Settings::load_from_file(&dir_path);
    assert!(result.is_err(), "Should fail to load from directory path");
    match result.unwrap_err() {
        MemorandaError::Io(io_err) => {
            // Should be a specific error about reading from directory
            let error_msg = io_err.to_string().to_lowercase();
            assert!(
                error_msg.contains("directory")
                    || error_msg.contains("is a directory")
                    || error_msg.contains("invalid")
                    || error_msg.contains("21"),
                "Error message should indicate directory issue: {error_msg}"
            );
        }
        MemorandaError::Json(_) => {} // Also acceptable if it tries to parse directory listing as JSON
        other => panic!("Expected IO or JSON error, got: {other:?}"),
    }
}

#[test]
fn test_settings_load_from_corrupted_file() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();

    // Write invalid JSON to file
    fs::write(&path, "{ invalid json content }").unwrap();

    let result = Settings::load_from_file(&path);
    assert!(result.is_err());

    // Write valid JSON but invalid settings structure
    fs::write(&path, r#"{"invalid": "structure"}"#).unwrap();

    let result = Settings::load_from_file(&path);
    assert!(result.is_err());
}

#[test]
fn test_settings_load_from_empty_file() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();

    // Write empty file - should return defaults
    fs::write(&path, "").unwrap();

    let result = Settings::load_from_file(&path);
    assert!(result.is_ok());
    let settings = result.unwrap();
    assert_eq!(settings.log_level, "info");

    // Write only whitespace - should return defaults
    fs::write(&path, "   \n\t  ").unwrap();

    let result = Settings::load_from_file(&path);
    assert!(result.is_ok());
    let settings = result.unwrap();
    assert_eq!(settings.log_level, "info");
}

#[test]
fn test_settings_load_from_large_file() {
    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();

    // Create a large but valid JSON file
    let large_data_dir = "x".repeat(10000);
    let large_settings = Settings {
        data_dir: PathBuf::from(large_data_dir),
        log_level: "info".to_string(),
        mcp_server_port: 8080,
        minimum_rust_version: "1.70.0".to_string(),
        max_memo_file_size: 1_000_000,
    };

    large_settings.save_to_file(&path).unwrap();
    let loaded_settings = Settings::load_from_file(&path).unwrap();
    assert_eq!(large_settings.data_dir, loaded_settings.data_dir);
}

#[test]
fn test_settings_serialization_roundtrip() {
    let original = Settings {
        data_dir: PathBuf::from("./test/data"),
        log_level: "debug".to_string(),
        mcp_server_port: 9090,
        minimum_rust_version: "1.75.0".to_string(),
        max_memo_file_size: 5_000_000,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&original).unwrap();

    // Deserialize back
    let deserialized: Settings = serde_json::from_str(&json).unwrap();

    assert_eq!(original.data_dir, deserialized.data_dir);
    assert_eq!(original.log_level, deserialized.log_level);
    assert_eq!(original.mcp_server_port, deserialized.mcp_server_port);
    assert_eq!(
        original.minimum_rust_version,
        deserialized.minimum_rust_version
    );
    assert_eq!(original.max_memo_file_size, deserialized.max_memo_file_size);
}

#[test]
fn test_settings_validation_with_realistic_scenarios() {
    // Test typical development environment settings
    let dev_settings = Settings {
        data_dir: PathBuf::from("./dev-data"),
        log_level: "debug".to_string(),
        mcp_server_port: 3000,
        minimum_rust_version: "1.70.0".to_string(),
        max_memo_file_size: 10_000_000, // 10MB
    };
    assert!(dev_settings.validate().is_ok());

    // Test production environment settings
    let prod_settings = Settings {
        data_dir: PathBuf::from("/var/lib/memoranda"),
        log_level: "info".to_string(),
        mcp_server_port: 8080,
        minimum_rust_version: "1.75.0".to_string(),
        max_memo_file_size: 100_000_000, // 100MB
    };
    assert!(prod_settings.validate().is_ok());

    // Test low-resource environment settings
    let low_resource_settings = Settings {
        data_dir: PathBuf::from("./minimal-data"),
        log_level: "error".to_string(),
        mcp_server_port: 8080,
        minimum_rust_version: "1.70.0".to_string(),
        max_memo_file_size: 1_000_000, // 1MB
    };
    assert!(low_resource_settings.validate().is_ok());
}

#[test]
fn test_settings_concurrent_file_operations() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;

    let temp_dir = TempDir::new().unwrap();
    let success_count = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    // Test concurrent save operations
    for i in 0..10 {
        let temp_dir = temp_dir.path().to_path_buf();
        let success_count = Arc::clone(&success_count);

        let handle = thread::spawn(move || {
            let settings = Settings {
                data_dir: PathBuf::from(format!("./data-{i}")),
                log_level: "info".to_string(),
                mcp_server_port: 8080 + i as u16,
                minimum_rust_version: "1.70.0".to_string(),
                max_memo_file_size: 1_000_000,
            };

            let path = temp_dir.join(format!("settings-{i}.json"));
            if settings.save_to_file(&path).is_ok() {
                success_count.fetch_add(1, Ordering::Relaxed);
            }
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // All operations should succeed
    assert_eq!(success_count.load(Ordering::Relaxed), 10);
}

#[test]
fn test_settings_path_handling_edge_cases() {
    // Test with relative paths
    let settings = Settings {
        data_dir: PathBuf::from("../data"),
        log_level: "info".to_string(),
        mcp_server_port: 8080,
        minimum_rust_version: "1.70.0".to_string(),
        max_memo_file_size: 1_000_000,
    };
    assert!(settings.validate().is_ok());

    // Test with absolute paths
    let settings = Settings {
        data_dir: PathBuf::from("/tmp/memoranda"),
        log_level: "info".to_string(),
        mcp_server_port: 8080,
        minimum_rust_version: "1.70.0".to_string(),
        max_memo_file_size: 1_000_000,
    };
    assert!(settings.validate().is_ok());

    // Test with paths containing special characters
    let settings = Settings {
        data_dir: PathBuf::from("./data with spaces & symbols!"),
        log_level: "info".to_string(),
        mcp_server_port: 8080,
        minimum_rust_version: "1.70.0".to_string(),
        max_memo_file_size: 1_000_000,
    };
    assert!(settings.validate().is_ok());
}

#[test]
fn test_settings_error_message_quality() {
    // Test that error messages are descriptive and helpful
    let settings = Settings {
        data_dir: PathBuf::from("./data"),
        log_level: "info".to_string(),
        mcp_server_port: 80, // Invalid port
        minimum_rust_version: "1.70.0".to_string(),
        max_memo_file_size: 1_000_000,
    };

    match settings.validate() {
        Err(MemorandaError::Validation { message }) => {
            assert!(message.contains("port"));
            assert!(message.contains("1024"));
            assert!(message.contains("80"));
        }
        _ => panic!("Expected validation error with descriptive message"),
    }

    // Test error message for invalid rust version
    let settings = Settings {
        data_dir: PathBuf::from("./data"),
        log_level: "info".to_string(),
        mcp_server_port: 8080,
        minimum_rust_version: "invalid".to_string(),
        max_memo_file_size: 1_000_000,
    };

    match settings.validate() {
        Err(MemorandaError::Validation { message }) => {
            assert!(message.contains("Invalid minimum Rust version"));
            assert!(message.contains("invalid"));
            assert!(message.contains("semver"));
        }
        _ => panic!("Expected validation error with descriptive message"),
    }
}

#[test]
fn test_settings_property_based_validation() {
    // Use property-based testing to verify settings validation
    for _ in 0..100 {
        let port: u16 = (1024..=65535).fake();
        let log_level: String = {
            let levels = ["error", "warn", "info", "debug", "trace"];
            let index: usize = (0..levels.len()).fake();
            levels[index].to_string()
        };
        let version = format!(
            "{}.{}.{}",
            (1..=2).fake::<u32>(),
            (0..=99).fake::<u32>(),
            (0..=99).fake::<u32>()
        );
        let max_size: u64 = (1..=u64::MAX).fake();

        let settings = Settings {
            data_dir: PathBuf::from("./data"),
            log_level,
            mcp_server_port: port,
            minimum_rust_version: version,
            max_memo_file_size: max_size,
        };

        assert!(settings.validate().is_ok());
    }
}

#[test]
fn test_settings_advanced_error_scenarios() {
    // Test validation errors with specific error types

    // Test invalid port boundary
    let settings = Settings {
        data_dir: PathBuf::from("./data"),
        log_level: "info".to_string(),
        mcp_server_port: 1023, // Just below minimum
        minimum_rust_version: "1.70.0".to_string(),
        max_memo_file_size: 1_000_000,
    };

    let result = settings.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Port must be"));
    assert!(error.to_string().contains("1024"));

    // Test invalid Rust version formats
    let invalid_versions = vec![
        "1.70",          // Missing patch version
        "1",             // Too short
        "1.70.0.1",      // Too many components
        "1.70.0-beta",   // Pre-release
        "1.70.0+build",  // Build metadata
        "v1.70.0",       // With prefix
        "not.a.version", // Invalid format
        "1.x.0",         // Invalid characters
    ];

    for version in invalid_versions {
        let settings = Settings {
            data_dir: PathBuf::from("./data"),
            log_level: "info".to_string(),
            mcp_server_port: 8080,
            minimum_rust_version: version.to_string(),
            max_memo_file_size: 1_000_000,
        };

        let result = settings.validate();
        assert!(result.is_err(), "Version '{version}' should be invalid");
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("Invalid minimum Rust version"),
            "Error message should mention invalid Rust version for '{version}'"
        );
    }

    // Test empty fields
    let settings = Settings {
        data_dir: PathBuf::from("./data"),
        log_level: "".to_string(), // Empty log level
        mcp_server_port: 8080,
        minimum_rust_version: "1.70.0".to_string(),
        max_memo_file_size: 1_000_000,
    };

    let result = settings.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Log level cannot be empty"));

    // Test zero file size
    let settings = Settings {
        data_dir: PathBuf::from("./data"),
        log_level: "info".to_string(),
        mcp_server_port: 8080,
        minimum_rust_version: "1.70.0".to_string(),
        max_memo_file_size: 0, // Invalid size
    };

    let result = settings.validate();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("must be at least"));
}

#[test]
fn test_settings_serialization_error_handling() {
    // Test handling of malformed JSON during deserialization
    use std::fs;
    use tempfile::NamedTempFile;

    let temp_file = NamedTempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();

    // Test various malformed JSON scenarios
    let malformed_json_cases = [
        "",                                          // Empty file
        "{",                                         // Unclosed brace
        "{ \"invalid\": }",                          // Missing value
        "{ \"data_dir\": true }",                    // Wrong type for field
        "{ \"mcp_server_port\": \"not_a_number\" }", // String where number expected
        "not json at all",                           // Not JSON
        "{ \"unknown_field_only\": true }",          // Missing required fields
    ];

    for (i, malformed_json) in malformed_json_cases.iter().enumerate() {
        fs::write(&path, malformed_json).unwrap();

        let result = Settings::load_from_file(&path);
        if malformed_json.is_empty() {
            // Empty file should return default settings
            assert!(
                result.is_ok(),
                "Case {i}: Empty file should return defaults"
            );
            let settings = result.unwrap();
            assert_eq!(settings.log_level, "info");
        } else {
            // All other malformed JSON should fail
            assert!(
                result.is_err(),
                "Case {i}: Malformed JSON '{malformed_json}' should fail to parse"
            );
        }
    }
}

#[test]
fn test_settings_filesystem_permission_scenarios() {
    use std::fs;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let settings = Settings::new().unwrap();

    // Create a read-only directory to test permission errors
    let readonly_dir = temp_dir.path().join("readonly");
    fs::create_dir(&readonly_dir).unwrap();

    // Set directory to read-only (this may not work on all systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&readonly_dir).unwrap().permissions();
        perms.set_mode(0o444); // Read-only
        let _ = fs::set_permissions(&readonly_dir, perms);

        // Try to save to read-only directory
        let readonly_path = readonly_dir.join("settings.json");
        let result = settings.save_to_file(&readonly_path);
        // This might or might not fail depending on system and user permissions
        // We just verify the method doesn't panic
        let _ = result;
    }

    // Test with very long path names that might exceed filesystem limits
    let long_filename = "a".repeat(300); // Very long filename
    let long_path = temp_dir.path().join(long_filename);
    let result = settings.save_to_file(&long_path);
    // This might succeed or fail depending on filesystem limits
    // We just verify no panic occurs
    let _ = result;
}

#[test]
fn test_settings_edge_case_values() {
    // Test with edge case but valid values

    // Test with maximum port value
    let settings = Settings {
        data_dir: PathBuf::from("./data"),
        log_level: "trace".to_string(), // Valid but uncommon level
        mcp_server_port: 65535,         // Maximum u16 value
        minimum_rust_version: "1.0.0".to_string(), // Very old but valid version
        max_memo_file_size: u64::MAX,   // Maximum possible file size
    };

    assert!(settings.validate().is_ok());

    // Test with minimum valid values
    let settings = Settings {
        data_dir: PathBuf::from("."),              // Minimal path
        log_level: "error".to_string(),            // Valid minimal level
        mcp_server_port: 1024,                     // Minimum valid port
        minimum_rust_version: "0.1.0".to_string(), // Very early version
        max_memo_file_size: 1,                     // Minimum file size
    };

    assert!(settings.validate().is_ok());

    // Test with unicode in paths
    let settings = Settings {
        data_dir: PathBuf::from("./数据/مجلد/папка"), // Unicode characters
        log_level: "info".to_string(),
        mcp_server_port: 8080,
        minimum_rust_version: "1.70.0".to_string(),
        max_memo_file_size: 1_000_000,
    };

    assert!(settings.validate().is_ok());
}
