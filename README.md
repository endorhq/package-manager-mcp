# Package Manager MCP

A Model Context Protocol (MCP) server that provides AI agents control over OS package managers.

## Overview

This MCP server enables AI agents to interact with Linux distributions by providing tools to manage packages through the OS package manager. It offers a secure, controlled interface for package operations including installation, searching, listing, and repository management.

## Features

- **Package Installation**: Install Linux distributions packages with optional custom repository support
- **Package Search**: Search for available packages in configured repositories
- **Package Listing**: List all currently installed packages
- **Repository Management**: Refresh package repository indexes
- **Error Handling**: Comprehensive error reporting with detailed feedback
- **Security**: Controlled execution environment with proper privilege handling

## Available Tools

### `install_package`
Install Linux distribution packages using the system package manager.
- **Parameters**:
  - `package_name` (required): Exact name of the package to install
  - `repository` (optional): Custom repository URL for package installation
- **Example**: Install curl from default repositories or a specific repository

### `search_package`
Search for packages by name or keyword.
- **Parameters**:
  - `query` (required): Search term for package names or descriptions
- **Example**: Search for all packages containing "python"

### `list_installed_packages`
List all currently installed packages on the system.
- **Parameters**: None
- **Returns**: Complete list of installed packages with versions

### `refresh_repositories`
Update package repository indexes to get latest package information.
- **Parameters**: None
- **Example**: Refresh all configured repositories before installing packages

## Installation

### Prerequisites

- Rust (2024 edition)
- Appropriate permissions to run package manager

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd package-manager-mcp

# Build the project
make build

# Or build for specific target
make TARGET=x86_64-unknown-linux-musl build
```

### Running the Server

```bash
# Run with default settings (host: 0.0.0.0, port: 8090)
./target/debug/package-manager-mcp

# Or specify custom host and port
./target/debug/package-manager-mcp --host 127.0.0.1 --port 3000
```

## Configuration

The server accepts the following command-line arguments:

- `--host`: Host address to bind to (default: 0.0.0.0)
- `--port`: Port number to listen on (default: 8090)

## Docker Usage

The project includes Docker support for testing in containerized environments:

```bash
# Run in Alpine container
make run-alpine

# Run with MCP inspector for testing
make inspector-alpine
```

## Development

### Code Formatting
```bash
make fmt          # Format Rust code
make nix-fmt      # Format Nix files
```

### Linting
```bash
make lint         # Run cargo clippy
```

### Testing
```bash
make test         # Run cargo tests
```

## MCP Integration

This server implements the Model Context Protocol (MCP) v2025-03-26 and can be integrated with any MCP-compatible AI client. The server provides:

- **Protocol Version**: 2025-03-26
- **Capabilities**: Tools enabled
- **Transport**: HTTP streaming with session management
- **Authentication**: Configurable (supports development mode)

### Example MCP Configuration

```json
{
  "mcpServers": {
    "package-manager": {
      "command": "./package-manager-mcp",
      "args": ["--host", "127.0.0.1", "--port", "8090"]
    }
  }
}
```

## Security Considerations

- The server executes package manager commands with the privileges of the running user
- Ensure proper user permissions and system security when deploying
- Package installations may require elevated privileges depending on system configuration
- Repository URLs are validated but should be from trusted sources

## Error Handling

The server provides comprehensive error handling with:
- Detailed error messages for failed operations
- Exit code reporting for debugging
- Stdout/stderr capture for troubleshooting
- Suggestions for common issues

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting: `make test lint`
5. Submit a pull request
