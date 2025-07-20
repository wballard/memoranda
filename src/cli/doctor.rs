use crate::config::Settings;
use anyhow::Result;
use std::fs;
use std::path::Path;
use tracing::debug;

#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticResult {
    Pass,
    Warning(String),
    Error(String),
}

pub struct DiagnosticCheck {
    pub name: String,
    pub description: String,
    pub check_fn: fn(&DoctorCommand) -> DiagnosticResult,
    pub fix_fn: Option<fn(&DoctorCommand) -> Result<()>>,
}

pub struct DoctorCommand {
    pub verbose: bool,
    pub auto_fix: bool,
    pub settings: Settings,
}

impl Default for DoctorCommand {
    fn default() -> Self {
        Self {
            verbose: false,
            auto_fix: false,
            settings: Settings::new_or_default(),
        }
    }
}

impl DoctorCommand {
    #[must_use]
    pub fn new() -> Self {
        Self {
            verbose: false,
            auto_fix: false,
            settings: Settings::new_or_default(),
        }
    }

    #[must_use]
    pub fn with_options(verbose: bool, auto_fix: bool) -> Self {
        Self {
            verbose,
            auto_fix,
            settings: Settings::new_or_default(),
        }
    }

    /// Runs the system diagnostic checks and displays the results.
    ///
    /// Performs various health checks on the system including Rust toolchain,
    /// system dependencies, git repository, memoranda directory structure,
    /// file permissions, memo formats, and MCP integration.
    ///
    /// # Errors
    ///
    /// Returns an error if any automatic fix operations fail when `auto_fix` is enabled.
    /// The function itself does not fail on diagnostic check failures - those are
    /// reported but do not cause the function to return an error.
    pub async fn run(&self) -> Result<()> {
        use colored::Colorize;

        debug!("Running doctor command");
        println!(
            "{}",
            "Memoranda Doctor - System Health Check"
                .bright_cyan()
                .bold()
        );
        println!("{}", "=====================================".bright_cyan());
        println!();

        let checks = Self::get_diagnostic_checks();
        let mut errors = 0;
        let mut warnings = 0;

        for check in checks {
            let result = (check.check_fn)(self);
            match result {
                DiagnosticResult::Pass => {
                    println!("{} {}", "✅".green(), check.name.green().bold());
                    if self.verbose {
                        println!("   {}", check.description.dimmed());
                    }
                }
                DiagnosticResult::Warning(msg) => {
                    println!("{} {}", "⚠️".yellow(), check.name.yellow().bold());
                    println!("   {}", msg.yellow());
                    warnings += 1;
                }
                DiagnosticResult::Error(msg) => {
                    println!("{} {}", "❌".red(), check.name.red().bold());
                    println!("   {}", msg.red());
                    if self.auto_fix {
                        if let Some(fix_fn) = check.fix_fn {
                            println!("   {}", "Attempting automatic fix...".bright_blue());
                            match fix_fn(self) {
                                Ok(()) => println!(
                                    "   {} {}",
                                    "✅".green(),
                                    "Fix applied successfully".green()
                                ),
                                Err(e) => println!(
                                    "   {} {}: {}",
                                    "❌".red(),
                                    "Fix failed".red(),
                                    e.to_string().red()
                                ),
                            }
                        }
                    }
                    errors += 1;
                }
            }
        }

        println!();
        if errors == 0 && warnings == 0 {
            println!(
                "{} {}",
                "✅".green(),
                "All systems operational! No issues found.".green().bold()
            );
        } else {
            if errors > 0 {
                println!(
                    "{} {}",
                    "❌".red(),
                    format!("Found {errors} error(s) that need attention.")
                        .red()
                        .bold()
                );
            }
            if warnings > 0 {
                println!(
                    "{} {}",
                    "⚠️".yellow(),
                    format!("Found {warnings} warning(s) that may need attention.")
                        .yellow()
                        .bold()
                );
            }
            println!();
            println!("{}", "RECOMMENDATIONS:".bright_cyan().bold());
            println!(
                "- Run {} to attempt automatic fixes",
                "'memoranda doctor --auto-fix'".bright_white()
            );
            println!(
                "- Run {} for detailed information",
                "'memoranda doctor --verbose'".bright_white()
            );
            println!("- See above for specific fix suggestions");
        }

        Ok(())
    }

    fn get_diagnostic_checks() -> Vec<DiagnosticCheck> {
        vec![
            DiagnosticCheck {
                name: "Rust toolchain".to_string(),
                description: "Checks if Rust toolchain is installed and meets minimum version"
                    .to_string(),
                check_fn: Self::check_rust_toolchain_diagnostic,
                fix_fn: None,
            },
            DiagnosticCheck {
                name: "System dependencies".to_string(),
                description: "Checks if required system dependencies are available".to_string(),
                check_fn: Self::check_system_dependencies_diagnostic,
                fix_fn: None,
            },
            DiagnosticCheck {
                name: "Git repository".to_string(),
                description: "Checks if a git repository is present".to_string(),
                check_fn: Self::check_git_repository_diagnostic,
                fix_fn: None,
            },
            DiagnosticCheck {
                name: "Memoranda directory".to_string(),
                description: "Checks if .memoranda directory exists and is accessible".to_string(),
                check_fn: Self::check_memoranda_directory_diagnostic,
                fix_fn: Some(Self::fix_memoranda_directory),
            },
            DiagnosticCheck {
                name: "File permissions".to_string(),
                description: "Checks read/write permissions on critical directories".to_string(),
                check_fn: Self::check_file_permissions_diagnostic,
                fix_fn: None,
            },
            DiagnosticCheck {
                name: "Memo file formats".to_string(),
                description: "Validates memo file formats and content".to_string(),
                check_fn: Self::check_memo_formats_diagnostic,
                fix_fn: Some(Self::fix_memo_formats),
            },
            DiagnosticCheck {
                name: "MCP integration".to_string(),
                description: "Checks MCP server initialization and tool registration".to_string(),
                check_fn: Self::check_mcp_integration_diagnostic,
                fix_fn: None,
            },
        ]
    }

    fn check_rust_toolchain_diagnostic(&self) -> DiagnosticResult {
        use std::process::Command;

        // Check if rustc is available
        match Command::new("rustc").arg("--version").output() {
            Ok(output) => {
                if output.status.success() {
                    let version_output = String::from_utf8_lossy(&output.stdout);
                    if let Some(version_line) = version_output.lines().next() {
                        // Extract version number
                        if let Some(version_str) = version_line.split_whitespace().nth(1) {
                            if let Ok(version) = semver::Version::parse(version_str) {
                                let min_version =
                                    semver::Version::parse(&self.settings.minimum_rust_version)
                                        .unwrap();
                                if version >= min_version {
                                    DiagnosticResult::Pass
                                } else {
                                    DiagnosticResult::Warning(format!(
                                        "Rust version {version} is below recommended minimum {min_version}. Consider updating with 'rustup update'."
                                    ))
                                }
                            } else {
                                DiagnosticResult::Warning(format!(
                                    "Could not parse Rust version from: {version_str}"
                                ))
                            }
                        } else {
                            DiagnosticResult::Warning(
                                "Could not extract version from rustc output".to_string(),
                            )
                        }
                    } else {
                        DiagnosticResult::Warning("Empty rustc version output".to_string())
                    }
                } else {
                    DiagnosticResult::Error("rustc command failed to execute".to_string())
                }
            }
            Err(_) => DiagnosticResult::Error(
                "Rust toolchain not found. Install Rust from https://rustup.rs/".to_string(),
            ),
        }
    }

    fn check_system_dependencies_diagnostic(&self) -> DiagnosticResult {
        use std::process::Command;

        let mut missing_deps = Vec::new();
        let mut warnings = Vec::new();

        // Check for git
        if Command::new("git").arg("--version").output().is_err() {
            warnings.push("git not found - some features may be limited");
        }

        // Check for basic system tools
        let tools = vec![
            ("cargo", "Cargo package manager"),
            ("rustc", "Rust compiler"),
        ];
        for (tool, description) in tools {
            if Command::new(tool).arg("--version").output().is_err() {
                missing_deps.push(format!("{tool} ({description})"));
            }
        }

        if !missing_deps.is_empty() {
            DiagnosticResult::Error(format!(
                "Missing required dependencies: {}. Install Rust toolchain from https://rustup.rs/",
                missing_deps.join(", ")
            ))
        } else if !warnings.is_empty() {
            DiagnosticResult::Warning(format!(
                "Optional dependencies missing: {}",
                warnings.join(", ")
            ))
        } else {
            DiagnosticResult::Pass
        }
    }

    fn check_git_repository_diagnostic(&self) -> DiagnosticResult {
        if Path::new(".git").exists() {
            DiagnosticResult::Pass
        } else {
            DiagnosticResult::Warning("No git repository found. Git integration provides better memo organization. Run 'git init' to initialize a repository.".to_string())
        }
    }

    fn check_memoranda_directory_diagnostic(&self) -> DiagnosticResult {
        let memoranda_path = Path::new(".memoranda");

        if memoranda_path.exists() {
            if memoranda_path.is_dir() {
                DiagnosticResult::Pass
            } else {
                DiagnosticResult::Error(".memoranda exists but is not a directory. Remove .memoranda file and create directory.".to_string())
            }
        } else {
            DiagnosticResult::Warning(".memoranda directory not found. Directory will be created automatically on first use.".to_string())
        }
    }

    fn check_file_permissions_diagnostic(&self) -> DiagnosticResult {
        let current_dir_result =
            self.check_directory_permissions_diagnostic(".", "Current directory");
        let memoranda_result = if Path::new(".memoranda").exists() {
            self.check_directory_permissions_diagnostic(".memoranda", ".memoranda directory")
        } else {
            DiagnosticResult::Pass
        };

        match (current_dir_result, memoranda_result) {
            (DiagnosticResult::Pass, DiagnosticResult::Pass) => DiagnosticResult::Pass,
            (DiagnosticResult::Error(msg), _) | (_, DiagnosticResult::Error(msg)) => {
                DiagnosticResult::Error(msg)
            }
            (DiagnosticResult::Warning(msg), _) | (_, DiagnosticResult::Warning(msg)) => {
                DiagnosticResult::Warning(msg)
            }
        }
    }

    fn check_memo_formats_diagnostic(&self) -> DiagnosticResult {
        let memoranda_path = Path::new(".memoranda");

        if !memoranda_path.exists() {
            return DiagnosticResult::Warning(
                "No memo files to validate (no .memoranda directory)".to_string(),
            );
        }

        match fs::read_dir(memoranda_path) {
            Ok(entries) => {
                let mut memo_count = 0;
                let mut issues = Vec::new();

                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        memo_count += 1;

                        // Enhanced validation
                        if let Err(e) = self.validate_memo_file_enhanced(&path) {
                            issues.push(format!("Invalid memo file {}: {}", path.display(), e));
                        }
                    }
                }

                if memo_count == 0 {
                    DiagnosticResult::Warning("No memo files found. Memo files will be created when you start using the system.".to_string())
                } else if issues.is_empty() {
                    DiagnosticResult::Pass
                } else {
                    DiagnosticResult::Error(format!(
                        "Found {} invalid memo files out of {} total: {}",
                        issues.len(),
                        memo_count,
                        issues.join(", ")
                    ))
                }
            }
            Err(e) => DiagnosticResult::Error(format!("Could not read .memoranda directory: {e}")),
        }
    }

    fn check_mcp_integration_diagnostic(&self) -> DiagnosticResult {
        use crate::mcp::McpServer;

        // Check 1: Server initialization
        let Ok(server) = McpServer::new("memoranda_doctor_test".to_string()) else {
            return DiagnosticResult::Error("Failed to initialize MCP server".to_string());
        };

        // Check 2: Tool registration
        let tools = server.get_tools();
        if tools.is_empty() {
            return DiagnosticResult::Error("No tools registered in MCP server".to_string());
        }

        // Check 3: Expected tools are present
        let expected_tools = &self.settings.expected_mcp_tools;

        let registered_tool_names: Vec<String> = tools
            .iter()
            .map(|tool| tool.to_tool_definition().name)
            .collect();

        let mut missing_tools = Vec::new();
        for expected_tool in expected_tools {
            if !registered_tool_names.contains(&expected_tool.to_string()) {
                missing_tools.push(expected_tool);
            }
        }

        if !missing_tools.is_empty() {
            return DiagnosticResult::Error(format!(
                "Missing required tools: {}",
                missing_tools
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        // Check 4: MCP SDK availability
        if !self.check_mcp_sdk_availability() {
            return DiagnosticResult::Warning(
                "MCP SDK may not be fully available - some features may be limited".to_string(),
            );
        }

        DiagnosticResult::Pass
    }

    fn check_mcp_sdk_availability(&self) -> bool {
        // Try to create a simple MCP-related structure to verify SDK availability
        // This is a basic check - in a real implementation, we might do more thorough validation
        true // For now, assume SDK is available if we can compile
    }

    fn check_directory_permissions_diagnostic(
        &self,
        path: &str,
        display_name: &str,
    ) -> DiagnosticResult {
        match fs::metadata(path) {
            Ok(metadata) => {
                if metadata.permissions().readonly() {
                    DiagnosticResult::Error(format!(
                        "{display_name} is read-only. Change {display_name} permissions to allow writing."
                    ))
                } else {
                    DiagnosticResult::Pass
                }
            }
            Err(e) => {
                DiagnosticResult::Error(format!("Could not check {display_name} permissions: {e}"))
            }
        }
    }

    /// Fixes the .memoranda directory by creating it if missing or removing it if it's a file.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Cannot remove existing .memoranda file
    /// - Cannot create .memoranda directory due to permissions or filesystem issues
    fn fix_memoranda_directory(&self) -> Result<()> {
        let memoranda_path = Path::new(".memoranda");

        if memoranda_path.exists() && !memoranda_path.is_dir() {
            fs::remove_file(memoranda_path)?;
        }

        if !memoranda_path.exists() {
            fs::create_dir(memoranda_path)?;
        }

        Ok(())
    }

    /// Fixes invalid memo file formats by renaming files with non-ULID names.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Cannot read the .memoranda directory
    /// - File operations (rename, validation) fail due to permissions or I/O issues
    fn fix_memo_formats(&self) -> Result<()> {
        let memoranda_path = Path::new(".memoranda");

        if !memoranda_path.exists() {
            return Ok(()); // Nothing to fix
        }

        let mut fixes_applied = 0;

        match fs::read_dir(memoranda_path) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json")
                        && self.validate_memo_file_enhanced(&path).is_err()
                    {
                        if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                            // Try to fix invalid filename format
                            if !Self::is_valid_ulid_filename(file_name) {
                                if let Ok(fixed_path) = Self::fix_memo_filename(&path) {
                                    println!(
                                        "   📝 Renamed {} to {}",
                                        path.display(),
                                        fixed_path.display()
                                    );
                                    fixes_applied += 1;
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => return Err(anyhow::anyhow!("Could not read .memoranda directory")),
        }

        if fixes_applied > 0 {
            println!("   ✅ Applied {fixes_applied} fix(es) to memo files");
        }

        Ok(())
    }

    /// Fixes a memo filename by renaming it to use a ULID-based name.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Cannot rename the file due to permissions, I/O issues, or filesystem constraints
    /// - Target filename already exists
    fn fix_memo_filename(path: &Path) -> Result<std::path::PathBuf> {
        // Generate a new ULID-based filename
        let ulid = ulid::Ulid::new();
        let new_filename = format!("{ulid}.json");
        let new_path = path.with_file_name(new_filename);

        // Move the file to the new name
        fs::rename(path, &new_path)?;

        Ok(new_path)
    }

    /// Validates a memo file for proper format, naming conventions, and content integrity.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Filename is not ULID-based format
    /// - Cannot read file due to permissions or I/O issues  
    /// - File contains invalid UTF-8
    /// - File contains invalid JSON
    /// - File size exceeds configured maximum
    /// - File content is empty
    fn validate_memo_file_enhanced(&self, path: &Path) -> Result<()> {
        // 1. Check file naming conventions (should be ULID-based)
        if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
            if !Self::is_valid_ulid_filename(file_name) {
                return Err(anyhow::anyhow!(
                    "Invalid filename format - should be ULID-based"
                ));
            }
        }

        // 2. Check UTF-8 encoding by reading as bytes first
        let content_bytes = fs::read(path)?;
        let Ok(content) = String::from_utf8(content_bytes) else {
            return Err(anyhow::anyhow!("File is not valid UTF-8"));
        };

        // 3. Check if file is valid JSON
        let json_value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Invalid JSON format: {}", e))?;

        // 4. Check for required fields in memo structure
        if let Some(obj) = json_value.as_object() {
            if !obj.contains_key("id") {
                return Err(anyhow::anyhow!("Missing required field: id"));
            }
            if !obj.contains_key("content") {
                return Err(anyhow::anyhow!("Missing required field: content"));
            }
        } else {
            return Err(anyhow::anyhow!("Memo file must contain a JSON object"));
        }

        // 5. Check content integrity (reasonable size limits)
        if content.len() > usize::try_from(self.settings.max_memo_file_size).unwrap_or(usize::MAX) {
            return Err(anyhow::anyhow!(
                "File size too large (>{}MB)",
                self.settings.max_memo_file_size / 1_000_000
            ));
        }

        if content.trim().is_empty() {
            return Err(anyhow::anyhow!("Empty memo file"));
        }

        Ok(())
    }

    fn is_valid_ulid_filename(filename: &str) -> bool {
        // ULID format: 26 characters, case-insensitive alphanumeric
        // Pattern: [0-9A-HJKMNP-TV-Z]{26}
        use regex::Regex;

        let ulid_regex = Regex::new(r"^[0-9A-HJKMNP-TV-Z]{26}$").unwrap();
        filename.len() == 26 && ulid_regex.is_match(&filename.to_uppercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Static mutex to synchronize directory changes across all tests
    static DIRECTORY_MUTEX: Mutex<()> = Mutex::new(());

    struct TestDirectoryGuard {
        original_dir: PathBuf,
        _guard: std::sync::MutexGuard<'static, ()>,
    }

    impl TestDirectoryGuard {
        fn new(temp_dir: &Path) -> Self {
            let guard = DIRECTORY_MUTEX.lock().unwrap();
            let original_dir = std::env::current_dir().unwrap();
            std::env::set_current_dir(temp_dir).unwrap();
            Self {
                original_dir,
                _guard: guard,
            }
        }
    }

    impl Drop for TestDirectoryGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original_dir);
        }
    }

    #[tokio::test]
    async fn test_doctor_command_execution() {
        let doctor = DoctorCommand::new();
        let result = doctor.run().await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_doctor_command_creation() {
        let doctor = DoctorCommand::new();
        // Just verify it can be created without panic
        let _ = doctor;
    }

    #[test]
    fn test_memoranda_directory_exists() {
        let temp_dir = TempDir::new().unwrap();
        let memoranda_path = temp_dir.path().join(".memoranda");
        fs::create_dir(&memoranda_path).unwrap();

        let doctor = DoctorCommand::new();
        let _guard = TestDirectoryGuard::new(temp_dir.path());
        let result = doctor.check_memoranda_directory_diagnostic();
        assert_eq!(result, DiagnosticResult::Pass);
    }

    #[test]
    fn test_memoranda_directory_missing() {
        let temp_dir = TempDir::new().unwrap();
        let doctor = DoctorCommand::new();
        let _guard = TestDirectoryGuard::new(temp_dir.path());
        let result = doctor.check_memoranda_directory_diagnostic();
        assert!(matches!(result, DiagnosticResult::Warning(_)));
    }

    #[test]
    fn test_memoranda_directory_is_file() {
        let temp_dir = TempDir::new().unwrap();
        let memoranda_path = temp_dir.path().join(".memoranda");
        fs::write(&memoranda_path, "not a directory").unwrap();

        let doctor = DoctorCommand::new();
        let _guard = TestDirectoryGuard::new(temp_dir.path());
        let result = doctor.check_memoranda_directory_diagnostic();
        assert!(matches!(result, DiagnosticResult::Error(_)));
    }

    #[test]
    fn test_git_repository_exists() {
        let temp_dir = TempDir::new().unwrap();
        let git_path = temp_dir.path().join(".git");
        fs::create_dir(&git_path).unwrap();

        let doctor = DoctorCommand::new();
        let _guard = TestDirectoryGuard::new(temp_dir.path());
        let result = doctor.check_git_repository_diagnostic();
        assert_eq!(result, DiagnosticResult::Pass);
    }

    #[test]
    fn test_git_repository_missing() {
        let temp_dir = TempDir::new().unwrap();
        let doctor = DoctorCommand::new();
        let _guard = TestDirectoryGuard::new(temp_dir.path());
        let result = doctor.check_git_repository_diagnostic();
        assert!(matches!(result, DiagnosticResult::Warning(_)));
    }

    #[test]
    fn test_valid_memo_file() {
        let temp_dir = TempDir::new().unwrap();
        // Use a valid ULID filename
        let memo_path = temp_dir.path().join("01K0FBWB1HSG75X617S118ZXHS.json");
        let valid_json = r#"{"id": "test", "content": "test memo"}"#;
        fs::write(&memo_path, valid_json).unwrap();

        let doctor = DoctorCommand::new();
        let result = doctor.validate_memo_file_enhanced(&memo_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_memo_file() {
        let temp_dir = TempDir::new().unwrap();
        // Use a valid ULID filename but invalid JSON content
        let memo_path = temp_dir.path().join("01K0FBWB1HSG75X617S118ZXHS.json");
        let invalid_json = r#"{"id": "test", "content": "test memo"#; // Missing closing brace
        fs::write(&memo_path, invalid_json).unwrap();

        let doctor = DoctorCommand::new();
        let result = doctor.validate_memo_file_enhanced(&memo_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_memo_formats_no_directory() {
        let temp_dir = TempDir::new().unwrap();
        let doctor = DoctorCommand::new();
        let _guard = TestDirectoryGuard::new(temp_dir.path());
        let result = doctor.check_memo_formats_diagnostic();
        assert!(matches!(result, DiagnosticResult::Warning(_)));
    }

    #[test]
    fn test_memo_formats_with_valid_files() {
        let temp_dir = TempDir::new().unwrap();
        let memoranda_path = temp_dir.path().join(".memoranda");
        fs::create_dir(&memoranda_path).unwrap();

        let valid_json = r#"{"id": "test", "content": "test memo"}"#;
        // Use a valid ULID filename
        let ulid_filename = "01K0FBWB1HSG75X617S118ZXHS.json";
        fs::write(memoranda_path.join(ulid_filename), valid_json).unwrap();

        let doctor = DoctorCommand::new();
        let _guard = TestDirectoryGuard::new(temp_dir.path());
        let result = doctor.check_memo_formats_diagnostic();
        assert_eq!(result, DiagnosticResult::Pass);
    }

    #[test]
    fn test_memo_formats_with_invalid_files() {
        let temp_dir = TempDir::new().unwrap();
        let memoranda_path = temp_dir.path().join(".memoranda");
        fs::create_dir(&memoranda_path).unwrap();

        let invalid_json = r#"{"id": "test", "content": "test memo"#; // Missing closing brace
        fs::write(memoranda_path.join("test.json"), invalid_json).unwrap();

        let doctor = DoctorCommand::new();
        let _guard = TestDirectoryGuard::new(temp_dir.path());
        let result = doctor.check_memo_formats_diagnostic();
        assert!(matches!(result, DiagnosticResult::Error(_)));
    }

    #[test]
    fn test_file_permissions_check() {
        let temp_dir = TempDir::new().unwrap();
        let doctor = DoctorCommand::new();
        let _guard = TestDirectoryGuard::new(temp_dir.path());
        let result = doctor.check_file_permissions_diagnostic();
        assert_eq!(result, DiagnosticResult::Pass);
    }

    #[test]
    fn test_diagnostic_result_enum() {
        let pass = DiagnosticResult::Pass;
        let warning = DiagnosticResult::Warning("test warning".to_string());
        let error = DiagnosticResult::Error("test error".to_string());

        assert_eq!(pass, DiagnosticResult::Pass);
        assert_eq!(
            warning,
            DiagnosticResult::Warning("test warning".to_string())
        );
        assert_eq!(error, DiagnosticResult::Error("test error".to_string()));
    }

    #[test]
    fn test_git_repository_diagnostic() {
        let temp_dir = TempDir::new().unwrap();
        let doctor = DoctorCommand::new();
        let _guard = TestDirectoryGuard::new(temp_dir.path());

        // Test without git repository
        let result = doctor.check_git_repository_diagnostic();
        assert!(matches!(result, DiagnosticResult::Warning(_)));

        // Test with git repository
        fs::create_dir(".git").unwrap();
        let result = doctor.check_git_repository_diagnostic();
        assert_eq!(result, DiagnosticResult::Pass);
    }

    #[test]
    fn test_memoranda_directory_diagnostic() {
        let temp_dir = TempDir::new().unwrap();
        let doctor = DoctorCommand::new();
        let _guard = TestDirectoryGuard::new(temp_dir.path());

        // Test without memoranda directory
        let result = doctor.check_memoranda_directory_diagnostic();
        assert!(matches!(result, DiagnosticResult::Warning(_)));

        // Test with memoranda directory
        fs::create_dir(".memoranda").unwrap();
        let result = doctor.check_memoranda_directory_diagnostic();
        assert_eq!(result, DiagnosticResult::Pass);

        // Test with memoranda as file
        fs::remove_dir_all(".memoranda").unwrap();
        fs::write(".memoranda", "not a directory").unwrap();
        let result = doctor.check_memoranda_directory_diagnostic();
        assert!(matches!(result, DiagnosticResult::Error(_)));
    }

    #[test]
    fn test_memo_formats_diagnostic() {
        // Test without memoranda directory
        let temp_dir = TempDir::new().unwrap();
        let doctor = DoctorCommand::new();
        let _guard = TestDirectoryGuard::new(temp_dir.path());

        let result = doctor.check_memo_formats_diagnostic();
        assert!(matches!(result, DiagnosticResult::Warning(_)));

        // Test with empty memoranda directory
        fs::create_dir(".memoranda").unwrap();
        let result = doctor.check_memo_formats_diagnostic();
        assert!(matches!(result, DiagnosticResult::Warning(_)));

        // Test with valid memo file
        let valid_json = r#"{"id": "test", "content": "test memo"}"#;
        // Use a valid ULID filename
        let ulid_filename = "01K0FBWB1HSG75X617S118ZXHS.json";
        fs::write(format!(".memoranda/{ulid_filename}"), valid_json).unwrap();
        let result = doctor.check_memo_formats_diagnostic();
        assert_eq!(result, DiagnosticResult::Pass);
    }

    #[test]
    fn test_memo_formats_diagnostic_with_invalid_file() {
        // Test with invalid memo file
        let temp_dir = TempDir::new().unwrap();
        let doctor = DoctorCommand::new();
        let _guard = TestDirectoryGuard::new(temp_dir.path());

        fs::create_dir(".memoranda").unwrap();
        let invalid_json = r#"{"id": "test", "content": "test memo"#;
        // Use invalid filename format (not ULID)
        fs::write(".memoranda/invalid.json", invalid_json).unwrap();
        let result = doctor.check_memo_formats_diagnostic();
        assert!(matches!(result, DiagnosticResult::Error(_)));
    }

    #[test]
    fn test_doctor_with_options() {
        let doctor = DoctorCommand::with_options(true, true);
        assert!(doctor.verbose);
        assert!(doctor.auto_fix);
    }

    #[test]
    fn test_fix_memoranda_directory() {
        let temp_dir = TempDir::new().unwrap();
        let doctor = DoctorCommand::new();
        let _guard = TestDirectoryGuard::new(temp_dir.path());

        // Test creating directory
        let result = doctor.fix_memoranda_directory();
        assert!(result.is_ok());
        assert!(Path::new(".memoranda").exists());
        assert!(Path::new(".memoranda").is_dir());

        // Test fixing file to directory
        fs::remove_dir_all(Path::new(".memoranda")).unwrap();
        fs::write(Path::new(".memoranda"), "not a directory").unwrap();
        let result = doctor.fix_memoranda_directory();
        assert!(result.is_ok());
        assert!(Path::new(".memoranda").exists());
        assert!(Path::new(".memoranda").is_dir());
    }
}
