use anyhow::Result;
use std::fs;
use std::path::Path;
use tracing::{info, warn};

#[derive(Default)]
pub struct DoctorCommand;

impl DoctorCommand {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self) -> Result<()> {
        info!("Running doctor command");
        println!("Memoranda Doctor - System Health Check");
        println!("=====================================");
        println!();

        let mut issues_found = 0;

        // Check .memoranda directory
        issues_found += self.check_memoranda_directory();

        // Check git repository
        issues_found += self.check_git_repository();

        // Check file permissions
        issues_found += self.check_file_permissions();

        // Check memo file formats
        issues_found += self.check_memo_formats();

        println!();
        if issues_found == 0 {
            println!("✅ All systems operational! No issues found.");
        } else {
            println!("❌ Found {issues_found} issue(s) that need attention.");
            println!();
            println!("RECOMMENDATIONS:");
            println!("- Run 'memoranda doctor' again after fixing issues");
            println!("- See above for specific fix suggestions");
        }

        Ok(())
    }

    fn check_memoranda_directory(&self) -> i32 {
        let memoranda_path = Path::new(".memoranda");

        if memoranda_path.exists() {
            if memoranda_path.is_dir() {
                println!("✅ .memoranda directory exists");
                0
            } else {
                println!("❌ .memoranda exists but is not a directory");
                println!("   Fix: Remove .memoranda file and create directory");
                1
            }
        } else {
            println!("⚠️  .memoranda directory not found");
            println!("   Fix: Directory will be created automatically on first use");
            0
        }
    }

    fn check_git_repository(&self) -> i32 {
        if Path::new(".git").exists() {
            println!("✅ Git repository detected");
            0
        } else {
            println!("⚠️  No git repository found");
            println!("   Info: Git integration provides better memo organization");
            println!("   Fix: Run 'git init' to initialize a repository");
            0
        }
    }

    fn check_directory_permissions(&self, path: &str, display_name: &str) -> i32 {
        match fs::metadata(path) {
            Ok(metadata) => {
                if metadata.permissions().readonly() {
                    println!("❌ {display_name} is read-only");
                    println!("   Fix: Change {display_name} permissions to allow writing");
                    1
                } else {
                    println!("✅ {display_name} permissions are correct");
                    0
                }
            }
            Err(e) => {
                warn!("Could not check {display_name} permissions: {e}");
                println!("❌ Could not check {display_name} permissions");
                1
            }
        }
    }

    fn check_file_permissions(&self) -> i32 {
        let mut issues = 0;

        // Check current directory write permissions
        issues += self.check_directory_permissions(".", "Current directory");

        // Check .memoranda directory permissions if it exists
        if Path::new(".memoranda").exists() {
            issues += self.check_directory_permissions(".memoranda", ".memoranda directory");
        }

        issues
    }

    fn check_memo_formats(&self) -> i32 {
        let memoranda_path = Path::new(".memoranda");

        if !memoranda_path.exists() {
            println!("⚠️  No memo files to validate (no .memoranda directory)");
            return 0;
        }

        match fs::read_dir(memoranda_path) {
            Ok(entries) => {
                let mut memo_count = 0;
                let mut issues = 0;

                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("json") {
                        memo_count += 1;
                        if let Err(e) = self.validate_memo_file(&path) {
                            println!("❌ Invalid memo file: {}", path.display());
                            println!("   Error: {e}");
                            issues += 1;
                        }
                    }
                }

                if memo_count == 0 {
                    println!("⚠️  No memo files found");
                    println!("   Info: Memo files will be created when you start using the system");
                } else if issues == 0 {
                    println!("✅ All {memo_count} memo files are valid");
                }

                issues
            }
            Err(e) => {
                warn!("Could not read .memoranda directory: {}", e);
                println!("❌ Could not read .memoranda directory");
                println!("   Error: {e}");
                1
            }
        }
    }

    fn validate_memo_file(&self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)?;
        let _: serde_json::Value = serde_json::from_str(&content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
