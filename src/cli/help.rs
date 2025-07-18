use anyhow::Result;
use tracing::info;

#[derive(Default)]
pub struct HelpCommand;

impl HelpCommand {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&self) -> Result<()> {
        info!("Displaying help");
        println!("Memoranda - A memory-augmented note-taking system");
        println!();
        println!("USAGE:");
        println!("    memoranda [COMMAND]");
        println!();
        println!("COMMANDS:");
        println!("    doctor    Check system health");
        println!("    help      Show this help message");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_command_execution() {
        let help = HelpCommand::new();
        let result = help.run();
        assert!(result.is_ok());
    }

    #[test]
    fn test_help_command_creation() {
        let help = HelpCommand::new();
        // Just verify it can be created without panic
        let _ = help;
    }
}

