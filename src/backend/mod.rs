pub mod apk;
pub mod apt;

use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler, model::*, service::RequestContext,
    tool_router,
};
use std::sync::Arc;

/// Result of executing a package manager command
pub struct ExecResult {
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub status: i32,
}

/// Options for installing a package
pub struct InstallOptions {
    pub package: String,
    pub repository: Option<String>,
}

/// Options for installing a package with a specific version
pub struct InstallVersionOptions {
    pub package: String,
    pub version: String,
}

/// Options for searching packages
pub struct SearchOptions {
    pub query: String,
    pub repository: Option<String>,
}

/// Trait defining the interface for package manager backends
pub trait PackageManager: Clone + Send + Sync + 'static {
    /// Returns the name of the package manager (e.g., "APK", "APT")
    fn name(&self) -> &'static str;

    /// Returns the OS name (e.g., "Alpine Linux", "Debian/Debian-derivative")
    fn os_name(&self) -> &'static str;

    /// Install a package (latest version)
    fn install_package(&self, options: &InstallOptions) -> Result<ExecResult, McpError>;

    /// Install a package with a specific version
    fn install_package_with_version(
        &self,
        options: &InstallVersionOptions,
    ) -> Result<ExecResult, McpError>;

    /// Search for packages
    fn search_package(&self, options: &SearchOptions) -> Result<ExecResult, McpError>;

    /// List installed packages
    fn list_installed_packages(&self) -> Result<ExecResult, McpError>;

    /// Refresh repository indexes
    fn refresh_repositories(&self) -> Result<ExecResult, McpError>;
}

/// Generic MCP handler that wraps any PackageManager implementation
#[derive(Clone)]
pub struct PackageManagerHandler<T: PackageManager> {
    backend: T,
}

#[tool_router]
impl<T: PackageManager> PackageManagerHandler<T> {
    pub fn new(backend: T) -> Self {
        Self { backend }
    }
}

impl<T: PackageManager> ServerHandler for PackageManagerHandler<T> {
    fn get_info(&self) -> ServerInfo {
        let instructions = format!(
            "This MCP server provides {} package management capabilities through the {} package manager. \
            Use this server to search for, install, update, list installed packages, and manage packages on {} systems. \
            The server executes {} commands with appropriate error handling and provides detailed feedback on operations.",
            self.backend.os_name(),
            self.backend.name(),
            self.backend.os_name(),
            self.backend.name()
        );

        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(instructions),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let pm_name = self.backend.name();
        let os_name = self.backend.os_name();
        let pm_lower = pm_name.to_lowercase();

        Ok(ListToolsResult {
            tools: vec![
                Tool {
                    name: "install_package".into(),
                    description: Some(std::borrow::Cow::Owned(format!(
                        "Install {} packages using the {} package manager. This tool executes '{}' commands with proper error handling. \
                        Use this when you need to install the latest version of software packages, libraries, or development tools on {} systems. \
                        If you need to install a specific version, use the install_package_with_version tool.",
                        os_name, pm_name,
                        if pm_lower == "apk" { "apk add" } else { "apt-get install" },
                        os_name
                    ))),
                    input_schema: Arc::new(
                        serde_json::from_value(serde_json::json!({
                            "type": "object",
                            "properties": {
                                "package_name": {
                                    "type": "string",
                                    "description": format!(
                                        "The exact name of the {} package to install (e.g., 'curl', 'python3', 'git'). \
                                        Package names are case-sensitive and should match the official package names in {} repositories. \
                                        Multiple packages can be specified by calling this tool multiple times.",
                                        os_name, os_name
                                    )
                                },
                                "repository": {
                                    "type": "string",
                                    "description": if pm_lower == "apk" {
                                        "Optional: Custom repository URL to use for package installation. Use this when you need to install packages from non-standard repositories or specific Alpine mirrors. Format should be a valid APK repository URL (e.g., 'https://dl-cdn.alpinelinux.org/alpine/edge/testing'). If not provided, the system's default configured repositories will be used.".to_string()
                                    } else {
                                        "Optional: Path to a custom sources.list file to use for package installation. If not provided, the system's default configured repositories will be used.".to_string()
                                    }
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
                    description: Some(std::borrow::Cow::Owned(format!(
                        "Install a specific version of a {os_name} package. This tool searches {os_name} repositories to find the requested package version, \
                        then installs it using exact version matching. Use this when you need to install a specific version of a package rather than the latest available version."
                    ))),
                    input_schema: Arc::new(
                        serde_json::from_value(serde_json::json!({
                            "type": "object",
                            "properties": {
                                "package_name": {
                                    "type": "string",
                                    "description": format!(
                                        "The exact name of the {} package to install (e.g., 'curl', 'python3', 'git'). \
                                        Package names are case-sensitive and should match the official package names in {} repositories.",
                                        os_name, os_name
                                    )
                                },
                                "version": {
                                    "type": "string",
                                    "description": format!(
                                        "The specific version of the package to install. The version string must match exactly as it appears in the repository. \
                                        If no exact match is found, the tool will return a list of available versions."
                                    )
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
                    description: Some(std::borrow::Cow::Owned(format!(
                        "Refresh registered repository indexes using '{}'. This tool synchronizes the local package database with remote repositories, \
                        ensuring you have access to the latest package information and versions. Use this before installing packages to get the most up-to-date package lists.",
                        if pm_lower == "apk" { "apk update" } else { "apt-get update" }
                    ))),
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
                    description: Some(std::borrow::Cow::Owned(format!(
                        "List all installed packages on {} using '{}'. This tool shows all packages currently installed on the system with their versions. \
                        Use this to audit installed software, check package versions, or verify installations.",
                        os_name,
                        if pm_lower == "apk" { "apk list -I" } else { "apt list --installed" }
                    ))),
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
                    description: Some(std::borrow::Cow::Owned(format!(
                        "Search for {} packages using the {} package manager. This tool executes '{}' commands to find packages matching your query. \
                        Use this when you need to discover available packages, find package names, or explore what software is available.",
                        os_name, pm_name,
                        if pm_lower == "apk" { "apk search" } else { "apt-cache search" }
                    ))),
                    input_schema: Arc::new(
                        serde_json::from_value(serde_json::json!({
                            "type": "object",
                            "properties": {
                                "query": {
                                    "type": "string",
                                    "description": format!(
                                        "Package name pattern to search for. Use exact package names (e.g., 'ruby', 'python3') or patterns to match multiple packages. \
                                        If you don't know the package name, try with specific package names first to avoid excessive output."
                                    )
                                },
                                "repository": {
                                    "type": "string",
                                    "description": if pm_lower == "apk" {
                                        "Optional: Specific repository URL to search in. If not provided, the search will query across multiple Alpine repositories (edge, v3.22, v3.21, v3.20, etc.) to find all available versions of matching packages.".to_string()
                                    } else {
                                        "Optional: This parameter is not used for APT searches. APT searches use the system's configured repositories.".to_string()
                                    }
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
        let pm_name = self.backend.name();
        let backend = self.backend.clone();

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
                    tokio::task::spawn_blocking(move || backend.install_package(&install_options))
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
                                format!("Package '{package}' was installed successfully.");
                            Ok(CallToolResult::success(vec![Content::text(
                                success_message,
                            )]))
                        } else {
                            let error_message = format!(
                                "Failed to install package '{package}' (exit code: {})",
                                exec_result.status
                            );
                            let mut error_details = serde_json::json!({
                                "package_name": package,
                                "exit_code": exec_result.status,
                                "package_manager": pm_name
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
                            "System error while installing package '{package}': {err:?}. This may indicate {pm_name} is not available or there are permission issues."
                        ),
                        Some(serde_json::json!({
                            "package_name": package,
                            "error_type": "system_error",
                            "suggestion": format!("Ensure {} package manager is installed and you have sufficient privileges", pm_name)
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

                let package_installation = tokio::task::spawn_blocking(move || {
                    backend.install_package_with_version(&install_version_options)
                })
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
                                "Package '{package}' version '{version}' was installed successfully."
                            );
                            Ok(CallToolResult::success(vec![Content::text(
                                success_message,
                            )]))
                        } else {
                            let error_message = format!(
                                "Failed to install package '{package}' version '{version}' (exit code: {})",
                                exec_result.status
                            );
                            let mut error_details = serde_json::json!({
                                "package_name": package,
                                "version": version,
                                "exit_code": exec_result.status,
                                "package_manager": pm_name
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
                    backend.refresh_repositories()
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
                                "All repositories were refreshed successfully.".to_string();
                            Ok(CallToolResult::success(vec![Content::text(
                                success_message,
                            )]))
                        } else {
                            let error_message = format!(
                                "Failed to refresh repositories (exit code: {})",
                                exec_result.status
                            );
                            let mut error_details = serde_json::json!({
                                "exit_code": exec_result.status,
                                "package_manager": pm_name
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
                            "System error while refreshing repositories: {err:?}. This may indicate {pm_name} is not available or there are permission issues."
                        ),
                        Some(serde_json::json!({
                            "error_type": "system_error",
                            "suggestion": format!("Ensure {} package manager is installed and you have sufficient privileges", pm_name)
                        })),
                    )),
                }
            }
            "list_installed_packages" => {
                let package_list =
                    tokio::task::spawn_blocking(move || backend.list_installed_packages())
                        .await
                        .map_err(|err| {
                            McpError::internal_error(
                                format!(
                                    "there was an error spawning package listing process: {err:?}"
                                ),
                                None,
                            )
                        })?;

                match package_list {
                    Ok(exec_result) => {
                        if exec_result.status == 0 {
                            let packages = exec_result.stdout.unwrap_or_default();
                            Ok(CallToolResult::success(vec![Content::text(format!(
                                "Installed packages:\n{packages}"
                            ))]))
                        } else {
                            let error_message = format!(
                                "Failed to list installed packages (exit code: {})",
                                exec_result.status
                            );
                            let mut error_details = serde_json::json!({
                                "exit_code": exec_result.status,
                                "package_manager": pm_name
                            });

                            if let Some(stderr) = exec_result.stderr {
                                error_details["stderr"] = serde_json::Value::String(stderr);
                            }

                            Err(McpError::internal_error(error_message, Some(error_details)))
                        }
                    }
                    Err(err) => Err(McpError::internal_error(
                        format!("System error while listing packages: {err:?}"),
                        Some(serde_json::json!({
                            "error_type": "system_error",
                            "suggestion": format!("Ensure {} package manager is available", pm_name)
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
                    repository,
                };

                let package_search = tokio::task::spawn_blocking(move || {
                    backend.search_package(&search_options)
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
                                        "Search completed for query '{query}' but no packages were found."
                                    )
                                } else {
                                    // Clean up `fetch` lines from APK output
                                    let cleaned_stdout = stdout
                                        .lines()
                                        .filter(|line| !line.starts_with("fetch "))
                                        .collect::<Vec<&str>>()
                                        .join("\n");

                                    format!(
                                        "Search results for query '{query}':\n\n{cleaned_stdout}"
                                    )
                                }
                            } else {
                                format!(
                                    "Search completed for query '{query}' but no packages were found."
                                )
                            };
                            Ok(CallToolResult::success(vec![Content::text(search_results)]))
                        } else {
                            let error_message = format!(
                                "Failed to search for packages with query '{query}' (exit code: {})",
                                exec_result.status
                            );
                            let mut error_details = serde_json::json!({
                                "query": query,
                                "exit_code": exec_result.status,
                                "package_manager": pm_name
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
                            "System error while searching for packages with query '{query}': {err:?}. This may indicate {pm_name} is not available or there are permission issues."
                        ),
                        Some(serde_json::json!({
                            "query": query,
                            "error_type": "system_error",
                            "suggestion": format!("Ensure {} package manager is installed and you have sufficient privileges", pm_name)
                        })),
                    )),
                }
            }
            _ => Ok(CallToolResult::error(vec![Content::text(format!(
                "Unknown tool '{}'. Available tools: install_package, install_package_with_version, list_installed_packages, refresh_repositories, search_package",
                request.name
            ))])),
        }
    }
}
