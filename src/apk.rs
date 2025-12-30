use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, model::*, service::RequestContext,
    tool_router,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct Apk {}

#[tool_router]
impl Apk {
    pub fn new() -> Self {
        Self {}
    }
}

impl ServerHandler for Apk {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This MCP server provides Alpine Linux package management capabilities through the APK package manager. Use this server to search for, install, update, list installed packages, and manage packages on Alpine Linux systems. The server executes APK commands with appropriate error handling and provides detailed feedback on operations.".to_string()),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult {
            tools: vec![
                Tool {
                    name: "install_package".into(),
                    description: Some(std::borrow::Cow::Borrowed("Install Alpine Linux packages using the APK package manager. This tool executes 'apk add' commands with proper error handling. Use this when you need to install the latest version of software packages, libraries, or development tools on Alpine Linux systems. If you need to install a specific version, use the install_package_with_version tool.")),
                    input_schema: Arc::new(
                        serde_json::from_value(serde_json::json!({
                            "type": "object",
                            "properties": {
                                "package_name": {
                                    "type": "string",
                                    "description": "The exact name of the Alpine Linux package to install (e.g., 'curl', 'python3', 'git'). Package names are case-sensitive and should match the official package names in Alpine repositories. Multiple packages can be specified by calling this tool multiple times."
                                },
                                "repository": {
                                    "type": "string",
                                    "description": "Optional: Custom repository URL to use for package installation. Use this when you need to install packages from non-standard repositories or specific Alpine mirrors. Format should be a valid APK repository URL (e.g., 'https://dl-cdn.alpinelinux.org/alpine/edge/testing'). If not provided, the system's default configured repositories will be used."
                                },
                            },
                            "required": ["package_name"]
                        })).map_err(|e| McpError::internal_error(format!("failed to parse install_package schema: {e}"), None))?,
                    ),
                    annotations: Some(ToolAnnotations {
                        idempotent_hint: Some(true),
                        open_world_hint: Some(true),
                        ..Default::default()
                    }),
                },
                Tool {
                    name: "install_package_with_version".into(),
                    description: Some(std::borrow::Cow::Borrowed("Install a specific version of an Alpine Linux package. This tool searches across multiple Alpine repositories to find the requested package version, then installs it using exact version matching with 'apk add package=version'. Use this when you need to install a specific version of a package rather than the latest available version.")),
                    input_schema: Arc::new(
                        serde_json::from_value(serde_json::json!({
                            "type": "object",
                            "properties": {
                                "package_name": {
                                    "type": "string",
                                    "description": "The exact name of the Alpine Linux package to install (e.g., 'curl', 'python3', 'git'). Package names are case-sensitive and should match the official package names in Alpine repositories."
                                },
                                "version": {
                                    "type": "string",
                                    "description": "The specific version of the package to install (e.g., '7.88.1-r1', '3.11.6-r0'). The version string must match exactly as it appears in the repository. If no exact match is found, the tool will return a list of available versions."
                                },
                            },
                            "required": ["package_name", "version"]
                        })).map_err(|e| McpError::internal_error(format!("failed to parse install_package_with_version schema: {e}"), None))?,
                    ),
                    annotations: Some(ToolAnnotations {
                        idempotent_hint: Some(true),
                        open_world_hint: Some(true),
                        ..Default::default()
                    }),
                },
                Tool {
                    name: "refresh_repositories".into(),
                    description: Some(std::borrow::Cow::Borrowed("Refresh registered repository indexes using 'apk update'. This tool synchronizes the local package database with remote repositories, ensuring you have access to the latest package information and versions. Use this before installing packages to get the most up-to-date package lists.")),
                    input_schema: Arc::new(
                        serde_json::from_value(serde_json::json!({
                            "type": "object",
                            "properties": {},
                            "required": []
                        })).map_err(|e| McpError::internal_error(format!("failed to parse refresh_repositories schema: {e}"), None))?,
                    ),
                    annotations: Some(ToolAnnotations {
                        idempotent_hint: Some(true),
                        open_world_hint: Some(true),
                        ..Default::default()
                    }),
                },
                Tool {
                    name: "list_installed_packages".into(),
                    description: Some(std::borrow::Cow::Borrowed("List all installed packages on Alpine Linux using 'apk list -I'. This tool shows all packages currently installed on the system with their versions and architectures. Use this to audit installed software, check package versions, or verify installations.")),
                    input_schema: Arc::new(
                        serde_json::from_value(serde_json::json!({
                            "type": "object",
                            "properties": {},
                            "required": []
                        })).map_err(|e| McpError::internal_error(format!("failed to parse list_installed_packages schema: {e}"), None))?,
                    ),
                    annotations: Some(ToolAnnotations {
                        idempotent_hint: Some(true),
                        open_world_hint: Some(false),
                        ..Default::default()
                    }),
                },
                Tool {
                    name: "search_package".into(),
                    description: Some(std::borrow::Cow::Borrowed("Search for Alpine Linux packages using the APK package manager. This tool executes 'apk search' commands to find packages matching your query. Use this when you need to discover available packages, find package names, or explore what software is available in the different alpine repositories.")),
                    input_schema: Arc::new(
                        serde_json::from_value(serde_json::json!({
                            "type": "object",
                            "properties": {
                                "query": {
                                    "type": "string",
                                    "description": "Package name pattern to search for. Use exact package names (e.g., 'ruby', 'python3') or wildcard patterns (e.g., 'ruby*', 'python*') to match multiple packages. If you don't know the package name, try with specific package names first to avoid excessive output and continue with wildcards if you don't find it."
                                },
                                "repository": {
                                    "type": "string",
                                    "description": "Optional: Specific repository URL to search in. If not provided, the search will query across multiple Alpine repositories (edge, v3.22, v3.21, v3.20, etc.) to find all available versions of matching packages."
                                },
                            },
                            "required": ["query"]
                        })).map_err(|e| McpError::internal_error(format!("failed to parse search_package schema: {e}"), None))?,
                    ),
                    annotations: Some(ToolAnnotations {
                        idempotent_hint: Some(true),
                        open_world_hint: Some(true),
                        ..Default::default()
                    }),
                }
            ],
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        match request.name.as_ref() {
            "install_package" => {
                let package = request
                    .arguments
                    .as_ref()
                    .and_then(|args| {
                        args.get("package_name")
                            .and_then(|package_name| package_name.as_str())
                    })
                    .ok_or_else(|| {
                        McpError::invalid_params("missing required parameter: package_name", None)
                    })?
                    .to_string();

                let repository = request
                    .arguments
                    .as_ref()
                    .and_then(|args| {
                        args.get("repository")
                            .and_then(|repository| repository.as_str())
                    })
                    .map(|repository| repository.to_string());

                let install_options = InstallOptions {
                    package: package.clone(),
                    repository: repository.clone(),
                };

                let package_installation =
                    tokio::task::spawn_blocking(move || install_package(&install_options))
                        .await
                        .map_err(|err| {
                            McpError::internal_error(
                                format!(
                                    "there was an error spawning installation process for package {package}: {err:?}"
                                ),
                                None,
                            )
                        })?;

                match package_installation {
                    Ok(exec_result) => {
                        if exec_result.status == 0 {
                            let success_message =
                                format!("✓ Package '{package}' was installed successfully.");
                            Ok(CallToolResult::success(vec![Content::text(
                                success_message,
                            )]))
                        } else {
                            let error_message = format!(
                                "✗ Failed to install package '{package}' (exit code: {})",
                                exec_result.status
                            );
                            let mut error_details = serde_json::json!({
                                "package_name": package,
                                "exit_code": exec_result.status,
                                "command": format!("apk add {}", if let Some(repo) = &repository { format!("--repository {repo} {package}") } else { package.clone() })
                            });

                            if let Some(stdout) = exec_result.stdout {
                                error_details["stdout"] = serde_json::Value::String(stdout);
                            }
                            if let Some(stderr) = exec_result.stderr {
                                error_details["stderr"] = serde_json::Value::String(stderr);
                            }

                            Err(McpError::internal_error(error_message, Some(error_details)))
                        }
                    }
                    Err(err) => Err(McpError::internal_error(
                        format!(
                            "✗ System error while installing package '{package}': {err:?}. This may indicate APK is not available or there are permission issues."
                        ),
                        Some(serde_json::json!({
                            "package_name": package,
                            "error_type": "system_error",
                            "suggestion": "Ensure APK package manager is installed and you have sufficient privileges"
                        })),
                    )),
                }
            }
            "install_package_with_version" => {
                let package = request
                    .arguments
                    .as_ref()
                    .and_then(|args| {
                        args.get("package_name")
                            .and_then(|package_name| package_name.as_str())
                    })
                    .ok_or_else(|| {
                        McpError::invalid_params("missing required parameter: package_name", None)
                    })?
                    .to_string();

                let version = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("version").and_then(|version| version.as_str()))
                    .ok_or_else(|| {
                        McpError::invalid_params("missing required parameter: version", None)
                    })?
                    .to_string();

                let install_version_options = InstallVersionOptions {
                    package: package.clone(),
                    version: version.clone(),
                };

                let package_installation =
                    tokio::task::spawn_blocking(move || install_package_with_version(&install_version_options))
                        .await
                        .map_err(|err| {
                            McpError::internal_error(
                                format!(
                                    "there was an error spawning installation process for package {package}={version}: {err:?}"
                                ),
                                None,
                            )
                        })?;

                match package_installation {
                    Ok(exec_result) => {
                        if exec_result.status == 0 {
                            let success_message = format!(
                                "✓ Package '{package}' version '{version}' was installed successfully."
                            );
                            Ok(CallToolResult::success(vec![Content::text(
                                success_message,
                            )]))
                        } else {
                            let error_message = format!(
                                "✗ Failed to install package '{package}' version '{version}' (exit code: {})",
                                exec_result.status
                            );
                            let mut error_details = serde_json::json!({
                                "package_name": package,
                                "version": version,
                                "exit_code": exec_result.status,
                                "command": format!("apk add {}={}", package, version)
                            });

                            if let Some(stdout) = exec_result.stdout {
                                error_details["stdout"] = serde_json::Value::String(stdout);
                            }
                            if let Some(stderr) = exec_result.stderr {
                                error_details["stderr"] = serde_json::Value::String(stderr);
                            }

                            Err(McpError::internal_error(error_message, Some(error_details)))
                        }
                    }
                    Err(err) => Err(err),
                }
            }
            "refresh_repositories" => {
                let repository_refresh = tokio::task::spawn_blocking(move || {
                    refresh_repositories()
                })
                .await
                .map_err(|err| {
                    McpError::internal_error(
                        format!("there was an error spawning repository refresh process: {err:?}"),
                        None,
                    )
                })?;

                match repository_refresh {
                    Ok(exec_result) => {
                        if exec_result.status == 0 {
                            let success_message =
                                "✓ All repositories were refreshed successfully.".to_string();
                            Ok(CallToolResult::success(vec![Content::text(
                                success_message,
                            )]))
                        } else {
                            let error_message = format!(
                                "✗ Failed to refresh repositories (exit code: {})",
                                exec_result.status
                            );
                            let mut error_details = serde_json::json!({
                                "exit_code": exec_result.status,
                                "command": "apk update".to_string()
                            });

                            if let Some(stdout) = exec_result.stdout {
                                error_details["stdout"] = serde_json::Value::String(stdout);
                            }
                            if let Some(stderr) = exec_result.stderr {
                                error_details["stderr"] = serde_json::Value::String(stderr);
                            }

                            Err(McpError::internal_error(error_message, Some(error_details)))
                        }
                    }
                    Err(err) => Err(McpError::internal_error(
                        format!(
                            "✗ System error while refreshing repositories: {err:?}. This may indicate APK is not available or there are permission issues."
                        ),
                        Some(serde_json::json!({
                            "error_type": "system_error",
                            "suggestion": "Ensure APK package manager is installed and you have sufficient privileges"
                        })),
                    )),
                }
            }
            "list_installed_packages" => {
                let package_list = tokio::task::spawn_blocking(list_installed_packages)
                    .await
                    .map_err(|err| {
                        McpError::internal_error(
                            format!("there was an error spawning package listing process: {err:?}"),
                            None,
                        )
                    })?;

                match package_list {
                    Ok(exec_result) => {
                        if exec_result.status == 0 {
                            let packages = exec_result.stdout.unwrap_or_default();
                            Ok(CallToolResult::success(vec![Content::text(format!(
                                "📦 Installed packages:\n{packages}"
                            ))]))
                        } else {
                            let error_message = format!(
                                "✗ Failed to list installed packages (exit code: {})",
                                exec_result.status
                            );
                            let mut error_details = serde_json::json!({
                                "exit_code": exec_result.status,
                                "command": "apk list -I"
                            });

                            if let Some(stderr) = exec_result.stderr {
                                error_details["stderr"] = serde_json::Value::String(stderr);
                            }

                            Err(McpError::internal_error(error_message, Some(error_details)))
                        }
                    }
                    Err(err) => Err(McpError::internal_error(
                        format!("✗ System error while listing packages: {err:?}"),
                        Some(serde_json::json!({
                            "error_type": "system_error",
                            "suggestion": "Ensure APK package manager is available"
                        })),
                    )),
                }
            }
            "search_package" => {
                let query = request
                    .arguments
                    .as_ref()
                    .and_then(|args| args.get("query").and_then(|query| query.as_str()))
                    .ok_or_else(|| {
                        McpError::invalid_params("missing required parameter: query", None)
                    })?
                    .to_string();

                let repository = request
                    .arguments
                    .as_ref()
                    .and_then(|args| {
                        args.get("repository")
                            .and_then(|repository| repository.as_str())
                    })
                    .map(|repository| repository.to_string());

                let search_options = SearchOptions {
                    query: query.clone(),
                    repository: repository.clone(),
                };

                let package_search = tokio::task::spawn_blocking(move || {
                    search_package(&search_options)
                })
                .await
                .map_err(|err| {
                    McpError::internal_error(
                        format!(
                            "there was an error spawning search process for query {query}: {err:?}"
                        ),
                        None,
                    )
                })?;

                match package_search {
                    Ok(exec_result) => {
                        if exec_result.status == 0 {
                            let search_results = if let Some(stdout) = exec_result.stdout {
                                if stdout.trim().is_empty() {
                                    format!(
                                        "✓ Search completed for query '{query}' but no packages were found."
                                    )
                                } else {
                                    // Clean up `fetch` lines from output
                                    let cleaned_stdout = stdout
                                        .lines()
                                        .filter(|line| !line.starts_with("fetch "))
                                        .collect::<Vec<&str>>()
                                        .join("\n");

                                    format!(
                                        "✓ Search results for query '{query}':\n\n{cleaned_stdout}"
                                    )
                                }
                            } else {
                                format!(
                                    "✓ Search completed for query '{query}' but no packages were found."
                                )
                            };
                            Ok(CallToolResult::success(vec![Content::text(search_results)]))
                        } else {
                            let error_message = format!(
                                "✗ Failed to search for packages with query '{query}' (exit code: {})",
                                exec_result.status
                            );
                            let mut error_details = serde_json::json!({
                                "query": query,
                                "exit_code": exec_result.status,
                                "command": format!("apk search {}", query.clone())
                            });

                            if let Some(stdout) = exec_result.stdout {
                                error_details["stdout"] = serde_json::Value::String(stdout);
                            }
                            if let Some(stderr) = exec_result.stderr {
                                error_details["stderr"] = serde_json::Value::String(stderr);
                            }

                            Err(McpError::internal_error(error_message, Some(error_details)))
                        }
                    }
                    Err(err) => Err(McpError::internal_error(
                        format!(
                            "✗ System error while searching for packages with query '{query}': {err:?}. This may indicate APK is not available or there are permission issues."
                        ),
                        Some(serde_json::json!({
                            "query": query,
                            "error_type": "system_error",
                            "suggestion": "Ensure APK package manager is installed and you have sufficient privileges"
                        })),
                    )),
                }
            }
            _ => Ok(CallToolResult::error(vec![Content::text(format!(
                "✗ Unknown tool '{}'. Available tools: install_package, install_package_with_version, list_installed_packages, refresh_repositories, search_package",
                request.name
            ))])),
        }
    }
}

struct InstallOptions {
    package: String,
    repository: Option<String>,
}

struct SearchOptions {
    query: String,
    repository: Option<String>,
}

struct InstallVersionOptions {
    package: String,
    version: String,
}

struct ExecResult {
    stdout: Option<String>,
    stderr: Option<String>,
    status: i32,
}

/// List of repositories to search across
const SEARCH_REPOSITORIES: &[&str] = &[
    "https://dl-cdn.alpinelinux.org/alpine/edge/main",
    "https://dl-cdn.alpinelinux.org/alpine/edge/community",
    // Current version
    "https://dl-cdn.alpinelinux.org/alpine/v3.22/main",
    "https://dl-cdn.alpinelinux.org/alpine/v3.22/community",
    // Older versions
    "https://dl-cdn.alpinelinux.org/alpine/v3.21/main",
    "https://dl-cdn.alpinelinux.org/alpine/v3.21/community",
    "https://dl-cdn.alpinelinux.org/alpine/v3.20/main",
    "https://dl-cdn.alpinelinux.org/alpine/v3.20/community",
    "https://dl-cdn.alpinelinux.org/alpine/v3.19/main",
    "https://dl-cdn.alpinelinux.org/alpine/v3.19/community",
    "https://dl-cdn.alpinelinux.org/alpine/v3.18/main",
    "https://dl-cdn.alpinelinux.org/alpine/v3.18/community",
    "https://dl-cdn.alpinelinux.org/alpine/v3.17/main",
    "https://dl-cdn.alpinelinux.org/alpine/v3.17/community",
    "https://dl-cdn.alpinelinux.org/alpine/v3.16/main",
    "https://dl-cdn.alpinelinux.org/alpine/v3.16/community",
    "https://dl-cdn.alpinelinux.org/alpine/v3.15/main",
    "https://dl-cdn.alpinelinux.org/alpine/v3.15/community",
];

fn install_package(install_options: &InstallOptions) -> Result<ExecResult, McpError> {
    let mut command = std::process::Command::new("apk");
    command.arg("add");

    if let Some(repository) = &install_options.repository {
        command.arg("--repository");
        command.arg(repository);
    }

    command.arg(&install_options.package);

    let command = command.output();

    let Ok(command) = command else {
        return Err(McpError::internal_error(
            format!(
                "there was an error installing package {}",
                &install_options.package
            ),
            None,
        ));
    };

    Ok(ExecResult {
        stdout: if !command.stdout.is_empty() {
            Some(String::from_utf8_lossy(&command.stdout).to_string())
        } else {
            None
        },
        stderr: if !command.stderr.is_empty() {
            Some(String::from_utf8_lossy(&command.stderr).to_string())
        } else {
            None
        },
        status: command.status.code().unwrap_or(-1),
    })
}

fn refresh_repositories() -> Result<ExecResult, McpError> {
    let mut command = std::process::Command::new("apk");
    command.arg("update");

    let command = command.output();

    let Ok(command) = command else {
        return Err(McpError::internal_error(
            "there was an error refreshing repositories".to_string(),
            None,
        ));
    };

    Ok(ExecResult {
        stdout: if !command.stdout.is_empty() {
            Some(String::from_utf8_lossy(&command.stdout).to_string())
        } else {
            None
        },
        stderr: if !command.stderr.is_empty() {
            Some(String::from_utf8_lossy(&command.stderr).to_string())
        } else {
            None
        },
        status: command.status.code().unwrap_or(-1),
    })
}

fn list_installed_packages() -> Result<ExecResult, McpError> {
    let command = std::process::Command::new("apk")
        .arg("list")
        .arg("-I")
        .output();

    let Ok(command) = command else {
        return Err(McpError::internal_error(
            "there was an error listing installed packages".to_string(),
            None,
        ));
    };

    Ok(ExecResult {
        stdout: if !command.stdout.is_empty() {
            Some(String::from_utf8_lossy(&command.stdout).to_string())
        } else {
            None
        },
        stderr: if !command.stderr.is_empty() {
            Some(String::from_utf8_lossy(&command.stderr).to_string())
        } else {
            None
        },
        status: command.status.code().unwrap_or(-1),
    })
}

fn search_package(search_options: &SearchOptions) -> Result<ExecResult, McpError> {
    let mut command = std::process::Command::new("apk");
    command.arg("--no-cache");

    // Add repositories: use provided repository or search all
    if let Some(repository) = &search_options.repository {
        command.arg("--repository");
        command.arg(repository);
    } else {
        // Search across all repositories
        for repo in SEARCH_REPOSITORIES {
            command.arg("--repository");
            command.arg(repo);
        }
    }

    command.arg("search");
    command.arg("--exact");
    command.arg("--all");
    command.arg(&search_options.query);

    let command = command.output();

    let Ok(command) = command else {
        return Err(McpError::internal_error(
            format!(
                "there was an error searching for packages with query {}",
                &search_options.query
            ),
            None,
        ));
    };

    Ok(ExecResult {
        stdout: if !command.stdout.is_empty() {
            Some(String::from_utf8_lossy(&command.stdout).to_string())
        } else {
            None
        },
        stderr: if !command.stderr.is_empty() {
            Some(String::from_utf8_lossy(&command.stderr).to_string())
        } else {
            None
        },
        status: command.status.code().unwrap_or(-1),
    })
}

fn validate_package_version_input(input: &str) -> bool {
    // Allow alphanumeric, dots, hyphens, underscores, and plus signs (common in version strings)
    input
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == '+')
}

fn install_package_with_version(options: &InstallVersionOptions) -> Result<ExecResult, McpError> {
    // Validate inputs to prevent command injection
    if !validate_package_version_input(&options.package) {
        return Err(McpError::internal_error(
            format!(
                "Invalid package name '{}': only alphanumeric characters, dots, hyphens, underscores, and plus signs are allowed",
                options.package
            ),
            Some(serde_json::json!({
                "package_name": options.package,
                "error_type": "validation_error"
            })),
        ));
    }

    if !validate_package_version_input(&options.version) {
        return Err(McpError::internal_error(
            format!(
                "Invalid version string '{}': only alphanumeric characters, dots, hyphens, underscores, and plus signs are allowed",
                options.version
            ),
            Some(serde_json::json!({
                "version": options.version,
                "error_type": "validation_error"
            })),
        ));
    }

    // Reuse the search_package function to find available versions
    let search_options = SearchOptions {
        query: options.package.clone(),
        repository: None, // Search across all repositories
    };

    let search_result = search_package(&search_options)?;

    // Parse the search output to find available versions
    let mut found_versions: Vec<String> = Vec::new();
    let mut version_found = false;

    if let Some(stdout) = &search_result.stdout {
        for line in stdout.lines() {
            // Skip fetch messages and empty lines
            if line.starts_with("fetch ") || line.trim().is_empty() {
                continue;
            }

            // Parse package-version from output
            // Format is typically: package-name-version
            if let Some(version_str) = line.strip_prefix(&format!("{}-", options.package)) {
                found_versions.push(version_str.to_string());

                // Check for exact version match
                if version_str == options.version {
                    version_found = true;
                }
            }
        }
    }

    // If exact version match found, install it
    if version_found {
        let mut install_cmd = std::process::Command::new("apk");
        install_cmd.arg("add");

        // Add all repositories - apk will find the right one
        for repo in SEARCH_REPOSITORIES {
            install_cmd.arg("--repository");
            install_cmd.arg(repo);
        }

        install_cmd.arg(format!("{}={}", options.package, options.version));

        let output = install_cmd.output().map_err(|err| {
            McpError::internal_error(
                format!(
                    "there was an error installing package {}={}: {}",
                    options.package, options.version, err
                ),
                None,
            )
        })?;

        return Ok(ExecResult {
            stdout: if !output.stdout.is_empty() {
                Some(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                None
            },
            stderr: if !output.stderr.is_empty() {
                Some(String::from_utf8_lossy(&output.stderr).to_string())
            } else {
                None
            },
            status: output.status.code().unwrap_or(-1),
        });
    }

    // Version not found - return error with available versions
    if found_versions.is_empty() {
        return Err(McpError::internal_error(
            format!(
                "Package '{}' not found in any searched repository",
                options.package
            ),
            Some(serde_json::json!({
                "package_name": options.package,
                "requested_version": options.version,
                "error_type": "package_not_found",
                "searched_repositories": SEARCH_REPOSITORIES
            })),
        ));
    }

    // Remove duplicates and sort available versions
    found_versions.sort();
    found_versions.dedup();

    Err(McpError::internal_error(
        format!(
            "Version '{}' of package '{}' not found. Available versions: {}",
            options.version,
            options.package,
            found_versions.join(", ")
        ),
        Some(serde_json::json!({
            "package_name": options.package,
            "requested_version": options.version,
            "available_versions": found_versions,
            "error_type": "version_not_found"
        })),
    ))
}
