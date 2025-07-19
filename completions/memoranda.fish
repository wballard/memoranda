# Fish completion for memoranda

# Main commands
complete -c memoranda -n "not __fish_seen_subcommand_from doctor serve help" -a "doctor" -d "Check system health and configuration"
complete -c memoranda -n "not __fish_seen_subcommand_from doctor serve help" -a "serve" -d "Start the MCP server"
complete -c memoranda -n "not __fish_seen_subcommand_from doctor serve help" -a "help" -d "Print help message or help for subcommand"

# Global options
complete -c memoranda -s h -l help -d "Print help"
complete -c memoranda -s V -l version -d "Print version"

# Doctor subcommand options
complete -c memoranda -n "__fish_seen_subcommand_from doctor" -l verbose -d "Show verbose output with detailed information"
complete -c memoranda -n "__fish_seen_subcommand_from doctor" -l auto-fix -d "Attempt to automatically fix issues"
complete -c memoranda -n "__fish_seen_subcommand_from doctor" -s h -l help -d "Print help"

# Serve subcommand options
complete -c memoranda -n "__fish_seen_subcommand_from serve" -s h -l help -d "Print help"

# Help subcommand completions
complete -c memoranda -n "__fish_seen_subcommand_from help" -a "doctor serve" -d "Get help for specific command"