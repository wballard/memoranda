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
        println!("Memoranda - A note-taking MCP server for coding agents");
        println!();
        println!("USAGE:");
        println!("    memoranda [COMMAND]");
        println!();
        println!("COMMANDS:");
        println!("    doctor    Check system health and configuration");
        println!("    serve     Start the MCP server on stdio");
        println!();
        println!("EXAMPLES:");
        println!("    memoranda doctor           # Run diagnostics");
        println!("    memoranda serve            # Start MCP server");
        println!();
        println!("MCP INTEGRATION:");
        println!("To use with Claude Code, add this to your MCP settings:");
        println!();
        println!("{{");
        println!("  \"mcpServers\": {{");
        println!("    \"memoranda\": {{");
        println!("      \"command\": \"memoranda\",");
        println!("      \"args\": [\"serve\"],");
        println!("      \"env\": {{}}");
        println!("    }}");
        println!("  }}");
        println!("}}");
        println!();
        println!("SETUP:");
        println!("1. Run 'memoranda doctor' to check your setup");
        println!("2. Add the MCP configuration above to Claude Code");
        println!("3. Use the memo tools in Claude Code to manage your notes");
        println!();
        println!("For more information, visit: https://github.com/wballard/memoranda");
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

