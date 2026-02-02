# AGENTS.md

This file provides guidance for AI agents working with code in this repository.

## Project Overview

This is an MCP (Model Context Protocol) server that provides AI agents with controlled access to Linux package managers. It supports multiple backends:
- **APK** for Alpine Linux
- **APT** for Debian/Debian-derivative

The server automatically detects the host OS at runtime and uses the appropriate backend. It's written in Rust and implements the MCP v2025-03-26 protocol specification.

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
make run-debian        # Run server in Debian container
make inspector-alpine  # Run with MCP inspector for Alpine testing
make inspector-debian  # Run with MCP inspector for Debian testing
```

The server runs on `0.0.0.0:8090` by default. Can be configured via command-line args:
```bash
./target/debug/package-manager-mcp --host 127.0.0.1 --port 3000
```

## Architecture

### Module Structure

```
src/
├── main.rs           # Entry point with OS detection and backend selection
├── backend/
│   ├── mod.rs        # PackageManager trait, shared types, generic ServerHandler
│   ├── apk.rs        # Alpine APK implementation
│   └── apt.rs        # Debian APT implementation
```

### Server Structure

The project follows a clean separation between the MCP protocol layer and the package manager implementations:

- **`src/main.rs`**: Entry point that sets up the HTTP server using Axum. Performs OS auto-detection via file system markers (`/etc/alpine-release` for Alpine, `/etc/debian_version` for Debian). Creates the appropriate `PackageManagerHandler<T>` and mounts it at `/mcp`.

- **`src/backend/mod.rs`**: Contains the shared infrastructure:
  - `ExecResult`, `InstallOptions`, `InstallVersionOptions`, `SearchOptions` - shared types
  - `PackageManager` trait - defines the interface all backends must implement
  - `PackageManagerHandler<T: PackageManager>` - generic MCP handler that implements `ServerHandler` once for all backends

- **`src/backend/apk.rs`**: Alpine Linux APK implementation:
  - `Apk` struct implementing `PackageManager` trait
  - Multi-repository search across Alpine edge, v3.22, v3.21, etc.
  - Version-specific installation with repository search

- **`src/backend/apt.rs`**: Debian/Debian-derivative APT implementation:
  - `Apt` struct implementing `PackageManager` trait
  - Uses `apt-get` with `DEBIAN_FRONTEND=noninteractive`
  - Version lookup via `apt-cache madison`

### Key Patterns

**Trait-based Abstraction**: The `PackageManager` trait defines a common interface:
```rust
pub trait PackageManager: Clone + Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn os_name(&self) -> &'static str;
    fn install_package(&self, options: &InstallOptions) -> Result<ExecResult, McpError>;
    fn install_package_with_version(&self, options: &InstallVersionOptions) -> Result<ExecResult, McpError>;
    fn search_package(&self, options: &SearchOptions) -> Result<ExecResult, McpError>;
    fn list_installed_packages(&self) -> Result<ExecResult, McpError>;
    fn refresh_repositories(&self) -> Result<ExecResult, McpError>;
}
```

**Generic Handler**: `PackageManagerHandler<T>` implements `ServerHandler` once, using the trait methods to delegate to the appropriate backend.

**Async Execution**: All package manager commands are executed in blocking tasks to avoid blocking the async runtime:
```rust
tokio::task::spawn_blocking(move || backend.install_package(&install_options))
```

**OS Auto-Detection**: Runtime detection via file system markers:
```rust
if std::path::Path::new("/etc/alpine-release").exists() {
    // Use APK backend
} else if std::path::Path::new("/etc/debian_version").exists() {
    // Use APT backend
}
```

**Error Handling**: Functions return `ExecResult` with stdout, stderr, and exit code. The handler checks exit codes and formats appropriate MCP errors with detailed context for troubleshooting.

**Tool Schema**: Input schemas are defined inline using `serde_json::json!` macros. Tool annotations include `idempotent_hint` and `open_world_hint` for MCP clients.

## MCP Tool Details

1. **install_package**: Installs packages (APK: `apk add`, APT: `apt-get install -y`)
2. **install_package_with_version**: Installs specific package version
3. **search_package**: Searches packages (APK: `apk search`, APT: `apt-cache search`)
4. **list_installed_packages**: Lists installed packages (APK: `apk list -I`, APT: `apt list --installed`)
5. **refresh_repositories**: Updates repository indexes (APK: `apk update`, APT: `apt-get update`)

All tools execute commands via `std::process::Command`, capture output, and return structured results.

## Important Notes

- This server supports Alpine Linux (APK) and Debian/Debian-derivative (APT) environments
- Requires appropriate permissions to execute package operations
- Uses Rust 2024 edition
- The inspector setup uses `DANGEROUSLY_OMIT_AUTH=true` for local development only
- The server includes graceful shutdown on Ctrl+C
