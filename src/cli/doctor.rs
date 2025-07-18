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
            println!("❌ Found {} issue(s) that need attention.", issues_found);
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

    fn check_file_permissions(&self) -> i32 {
        let mut issues = 0;
        
        // Check current directory write permissions
        match fs::metadata(".") {
            Ok(metadata) => {
                if metadata.permissions().readonly() {
                    println!("❌ Current directory is read-only");
                    println!("   Fix: Change directory permissions to allow writing");
                    issues += 1;
                } else {
                    println!("✅ Directory permissions are correct");
                }
            }
            Err(e) => {
                warn!("Could not check directory permissions: {}", e);
                println!("❌ Could not check directory permissions");
                issues += 1;
            }
        }

        // Check .memoranda directory permissions if it exists
        if Path::new(".memoranda").exists() {
            match fs::metadata(".memoranda") {
                Ok(metadata) => {
                    if metadata.permissions().readonly() {
                        println!("❌ .memoranda directory is read-only");
                        println!("   Fix: Change .memoranda directory permissions");
                        issues += 1;
                    } else {
                        println!("✅ .memoranda directory permissions are correct");
                    }
                }
                Err(e) => {
                    warn!("Could not check .memoranda permissions: {}", e);
                    println!("❌ Could not check .memoranda permissions");
                    issues += 1;
                }
            }
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

                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("json") {
                            memo_count += 1;
                            if let Err(e) = self.validate_memo_file(&path) {
                                println!("❌ Invalid memo file: {}", path.display());
                                println!("   Error: {}", e);
                                issues += 1;
                            }
                        }
                    }
                }

                if memo_count == 0 {
                    println!("⚠️  No memo files found");
                    println!("   Info: Memo files will be created when you start using the system");
                } else if issues == 0 {
                    println!("✅ All {} memo files are valid", memo_count);
                }

                issues
            }
            Err(e) => {
                warn!("Could not read .memoranda directory: {}", e);
                println!("❌ Could not read .memoranda directory");
                println!("   Error: {}", e);
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

