use anyhow::Result;
use tracing::info;

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