# memoranda

`memoranda` is an MCP console server implemented in Rust with the [MCP SDK](https://github.com/modelcontextprotocol/rust-sdk)

## Technology

- [MCP SDK](https://github.com/modelcontextprotocol/rust-sdk)

## Concepts

Memoranda provides the ability for coding agents to take simple text notes while working.
This is used to keep track of important discoveries and standards in the code base that serve as extended context for later coding agent tasks.

## Guidelines

Search the web for other MCP servers in Rust, in particular focusing on ones with many GitHub stars.
Think deeply about what makes them popular and take that into account we building the plan.

Create a great rust CLI, seeking inspirations from other popular rust based CLI programs such as `uv`.
Go above and beyond and exceed user expectations.

## Requirements

### CLI

As a user, I will use memoranda as a cli

### MCP

As a user, I will add memoranda as a local stdio MCP server as documented at https://docs.anthropic.com/en/docs/claude-code/mcp.

As a user, if I run memoranda stand alone in the shell, it will give me help information on how to add it to Claude Code.

### Memos in .memoranda

As a user, I will create memos as simple markdown `.md` files in directories using my own editor.

These will go in directories named `.memoranda`.

Each git repository will have a root .memoranda much like we have a root .claude or .github.

### Recording Memos

As a coding agent LLM, I want to use an MCP tool to create a memory either as:

- a new memory as a .md file in .memoranda
- a new memory adding to an exisitng file in .memoranda
- replacing or correcting a memory by editing an existing file in .memoranda

### Using Memos

As a coding agent LLM, I want to combine all available files into .memoranda and add it to my working context.


As a user, I want my arguments to NOT be required by default.

### Doctor

As a user, I want to be able to say `memoranda doctor` and get a diagnosis of any problem or error I need to correct.
