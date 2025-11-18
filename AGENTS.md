# AGENTS.md

This file provides guidance for AI agents working with code in this repository.

## Project Overview

This is an MCP (Model Context Protocol) server that provides AI agents with controlled access to the Alpine Linux APK package manager. It's written in Rust and implements the MCP v2025-03-26 protocol specification.

## Development Commands

### Building
```bash
make build                                  # Build for default target (x86_64-unknown-linux-musl)
make TARGET=aarch64-unknown-linux-musl build  # Build for specific target
```

### Code Quality
```bash
make fmt        # Format Rust code
make nix-fmt    # Format Nix files
make lint       # Run clippy
make test       # Run cargo tests
```

### Testing with Docker
```bash
make run-alpine        # Run server in Alpine container
make inspector-alpine  # Run with MCP inspector for interactive testing
```

The server runs on `0.0.0.0:8090` by default. Can be configured via command-line args:
```bash
./target/debug/package-manager-mcp --host 127.0.0.1 --port 3000
```

## Architecture

### Server Structure

The project follows a clean separation between the MCP protocol layer and the package manager implementation:

- **`src/main.rs`**: Entry point that sets up the HTTP server using Axum. Creates the `StreamableHttpService` with local session management and mounts it at `/mcp`. Uses clap for CLI argument parsing (host, port).

- **`src/apk.rs`**: Contains the core MCP implementation:
  - `Apk` struct implements `ServerHandler` trait from rmcp
  - Uses `#[tool_router]` macro for routing tool calls
  - Four MCP tools: `install_package`, `search_package`, `list_installed_packages`, `refresh_repositories`
  - Each tool spawns blocking operations using `tokio::task::spawn_blocking` since APK commands are synchronous
  - Comprehensive error handling with structured JSON error data including exit codes, stdout/stderr

### Key Patterns

**Async Execution**: All APK commands are executed in blocking tasks to avoid blocking the async runtime:
```rust
tokio::task::spawn_blocking(move || install_package(&install_options))
```

**Error Handling**: Functions return `ExecResult` with stdout, stderr, and exit code. The handler checks exit codes and formats appropriate MCP errors with detailed context for troubleshooting.

**Tool Schema**: Input schemas are defined inline using `serde_json::json!` macros. Tool annotations include `idempotent_hint` and `open_world_hint` for MCP clients.

**Protocol Implementation**: Uses the `rmcp` crate for MCP protocol handling. The server declares protocol version `V_2025_03_26` and tools capability.

## MCP Tool Details

1. **install_package**: Installs packages with `apk add`, supports optional custom repositories
2. **search_package**: Searches packages with `apk search`
3. **list_installed_packages**: Lists installed packages with `apk list -I`
4. **refresh_repositories**: Updates repository indexes with `apk update`

All tools execute APK commands via `std::process::Command`, capture output, and return structured results.

## Important Notes

- This server is designed for Alpine Linux environments (uses APK package manager)
- Requires appropriate permissions to execute package operations
- Uses Rust 2024 edition
- The inspector setup uses `DANGEROUSLY_OMIT_AUTH=true` for local development only
- The server includes graceful shutdown on Ctrl+C
