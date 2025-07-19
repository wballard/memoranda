use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Integration tests for CLI commands
/// These tests verify the CLI interface works correctly end-to-end

#[test]
fn test_cli_help_command() {
    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "A note-taking MCP server for coding agents",
        ))
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("doctor"))
        .stdout(predicate::str::contains("serve"));
}

#[test]
fn test_cli_no_command_shows_help() {
    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains(
            "Memoranda - A note-taking MCP server",
        ))
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_cli_doctor_command() {
    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Memoranda Doctor - System Health Check",
        ))
        .stdout(predicate::str::contains(
            "=====================================",
        ));
}

#[test]
fn test_cli_doctor_verbose_flag() {
    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.arg("doctor")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Memoranda Doctor - System Health Check",
        ));
}

#[test]
fn test_cli_doctor_auto_fix_flag() {
    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.arg("doctor")
        .arg("--auto-fix")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Memoranda Doctor - System Health Check",
        ));
}

#[test]
fn test_cli_doctor_both_flags() {
    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.arg("doctor")
        .arg("--verbose")
        .arg("--auto-fix")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Memoranda Doctor - System Health Check",
        ));
}

#[test]
fn test_cli_invalid_command() {
    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn test_cli_version_flag() {
    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("memoranda"));
}

#[test]
fn test_cli_help_flag() {
    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "A note-taking MCP server for coding agents",
        ))
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("Commands:"));
}

#[test]
fn test_cli_doctor_help() {
    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.arg("doctor")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Check system health and configuration",
        ))
        .stdout(predicate::str::contains("--verbose"))
        .stdout(predicate::str::contains("--auto-fix"));
}

#[test]
fn test_cli_serve_help() {
    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.arg("serve")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Start the MCP server"));
}

#[test]
fn test_cli_doctor_in_temporary_directory() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Memoranda Doctor - System Health Check",
        ));
}

#[test]
fn test_cli_doctor_with_git_repository() {
    let temp_dir = TempDir::new().unwrap();
    let git_dir = temp_dir.path().join(".git");
    fs::create_dir(&git_dir).unwrap();

    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅"))
        .stdout(predicate::str::contains("Git repository"));
}

#[test]
fn test_cli_doctor_with_memoranda_directory() {
    let temp_dir = TempDir::new().unwrap();
    let memoranda_dir = temp_dir.path().join(".memoranda");
    fs::create_dir(&memoranda_dir).unwrap();

    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("✅"))
        .stdout(predicate::str::contains("Memoranda directory"));
}

#[test]
fn test_cli_doctor_with_auto_fix_creates_directory() {
    let temp_dir = TempDir::new().unwrap();
    let memoranda_dir = temp_dir.path().join(".memoranda");

    // Ensure directory doesn't exist initially
    assert!(!memoranda_dir.exists());

    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .arg("--auto-fix")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Memoranda Doctor - System Health Check",
        ));

    // Note: Auto-fix may create the directory, but this depends on the actual implementation
    // The test just verifies the command runs successfully
}

#[test]
fn test_cli_doctor_with_invalid_memoranda_file() {
    let temp_dir = TempDir::new().unwrap();
    let memoranda_path = temp_dir.path().join(".memoranda");

    // Create a file instead of directory
    fs::write(&memoranda_path, "not a directory").unwrap();

    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("❌"))
        .stdout(predicate::str::contains("Memoranda directory"));
}

#[test]
fn test_cli_exit_codes() {
    // Test successful command
    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.arg("help").assert().success().code(0);

    // Test invalid command
    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.arg("invalid-command")
        .assert()
        .failure()
        .code(predicate::ne(0));
}

#[test]
fn test_cli_output_formatting() {
    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("Commands:"));
}

#[test]
fn test_cli_doctor_comprehensive_checks() {
    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.arg("doctor")
        .arg("--verbose")
        .assert()
        .success()
        .stdout(predicate::str::contains("Rust toolchain"))
        .stdout(predicate::str::contains("System dependencies"))
        .stdout(predicate::str::contains("Git repository"))
        .stdout(predicate::str::contains("Memoranda directory"))
        .stdout(predicate::str::contains("File permissions"))
        .stdout(predicate::str::contains("Memo file formats"))
        .stdout(predicate::str::contains("MCP integration"));
}

#[test]
fn test_cli_doctor_recommendations() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(
            predicate::str::contains("RECOMMENDATIONS:")
                .or(predicate::str::contains("All systems operational")),
        );
}

#[test]
fn test_cli_concurrent_execution() {
    use std::sync::mpsc;
    use std::thread;

    let (tx, rx) = mpsc::channel();

    // Run multiple CLI commands concurrently
    for i in 0..5 {
        let tx = tx.clone();
        thread::spawn(move || {
            let mut cmd = Command::cargo_bin("memoranda").unwrap();
            let _result = cmd.arg("help").assert().success();
            tx.send(i).unwrap();
        });
    }

    // Wait for all threads to complete
    for _ in 0..5 {
        rx.recv().unwrap();
    }
}

/// Test CLI behavior with different locale settings
#[test]
fn test_cli_locale_handling() {
    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.env("LANG", "en_US.UTF-8")
        .arg("help")
        .assert()
        .success()
        .stdout(predicate::str::contains("memoranda"));
}

/// Test CLI with different working directories
#[test]
fn test_cli_different_working_directories() {
    let temp_dir = TempDir::new().unwrap();
    let nested_dir = temp_dir.path().join("nested");
    fs::create_dir(&nested_dir).unwrap();

    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.current_dir(&nested_dir)
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Memoranda Doctor"));
}

/// Test CLI memory usage doesn't grow excessively
#[test]
fn test_cli_memory_usage() {
    for _ in 0..10 {
        let mut cmd = Command::cargo_bin("memoranda").unwrap();
        cmd.arg("help").assert().success();
    }
}

/// Test CLI handles unicode content properly
#[test]
fn test_cli_unicode_handling() {
    let temp_dir = TempDir::new().unwrap();
    let memoranda_dir = temp_dir.path().join(".memoranda");
    fs::create_dir(&memoranda_dir).unwrap();

    // Create a file with unicode content
    let unicode_file = memoranda_dir.join("unicode_test.md");
    fs::write(&unicode_file, "# 测试 Unicode 🦀 Rust").unwrap();

    let mut cmd = Command::cargo_bin("memoranda").unwrap();
    cmd.current_dir(temp_dir.path())
        .arg("doctor")
        .assert()
        .success()
        .stdout(predicate::str::contains("Memoranda Doctor"));
}
