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
            instructions: Some("This MCP server provides Alpine Linux package management capabilities through the APK package manager. Use this server to install, update, and manage packages on Alpine Linux systems. The server executes APK commands with appropriate error handling and provides detailed feedback on operations.".to_string()),
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
                    description: Some(std::borrow::Cow::Borrowed("Install Alpine Linux packages using the APK package manager. This tool executes 'apk add' commands with proper error handling. Use this when you need to install software packages, libraries, or development tools on Alpine Linux systems. The tool supports both official Alpine repositories and custom repository URLs.")),
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
                        })).unwrap(),
                    ),
                    annotations: Some(ToolAnnotations {
                        idempotent_hint: Some(true),
                        open_world_hint: Some(true),
                        ..Default::default()
                    }),
                },
                Tool {
                    name: "refresh_repositories".into(),
                    description: Some(std::borrow::Cow::Borrowed("Refresh Alpine Linux package repository indexes using 'apk update'. This tool synchronizes the local package database with remote repositories, ensuring you have access to the latest package information and versions. Use this before installing packages to get the most up-to-date package lists.")),
                    input_schema: Arc::new(
                        serde_json::from_value(serde_json::json!({
                            "type": "object",
                            "properties": {
                                "repository": {
                                    "type": "string",
                                    "description": "Optional: Custom repository URL to refresh. Use this when you need to update a specific repository rather than all configured repositories. Format should be a valid APK repository URL (e.g., 'https://dl-cdn.alpinelinux.org/alpine/edge/testing'). If not provided, all configured repositories will be refreshed."
                                },
                            },
                            "required": []
                        })).unwrap(),
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
                    .expect("mandatory argument")
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
            "refresh_repositories" => {
                let repository = request
                    .arguments
                    .as_ref()
                    .and_then(|args| {
                        args.get("repository")
                            .and_then(|repository| repository.as_str())
                    })
                    .map(|repository| repository.to_string());

                let refresh_options = RefreshOptions {
                    repository: repository.clone(),
                };

                let repository_refresh = tokio::task::spawn_blocking(move || {
                    refresh_repository(&refresh_options)
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
                            let success_message = if let Some(repo) = &repository {
                                format!("✓ Repository '{}' was refreshed successfully.", repo)
                            } else {
                                "✓ All repositories were refreshed successfully.".to_string()
                            };
                            Ok(CallToolResult::success(vec![Content::text(
                                success_message,
                            )]))
                        } else {
                            let error_message = if let Some(repo) = &repository {
                                format!(
                                    "✗ Failed to refresh repository '{}' (exit code: {})",
                                    repo, exec_result.status
                                )
                            } else {
                                format!(
                                    "✗ Failed to refresh repositories (exit code: {})",
                                    exec_result.status
                                )
                            };
                            let mut error_details = serde_json::json!({
                                "exit_code": exec_result.status,
                                "command": if let Some(repo) = &repository {
                                    format!("apk update --repository {}", repo)
                                } else {
                                    "apk update".to_string()
                                }
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
            _ => Ok(CallToolResult::error(vec![Content::text(format!(
                "✗ Unknown tool '{}'. Available tools: install_package, refresh_repositories",
                request.name
            ))])),
        }
    }
}

struct InstallOptions {
    package: String,
    repository: Option<String>,
}

struct RefreshOptions {
    repository: Option<String>,
}

struct ExecResult {
    stdout: Option<String>,
    stderr: Option<String>,
    status: i32,
}

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
        status: command.status.code().expect("exit code is expected"),
    })
}

fn refresh_repository(refresh_options: &RefreshOptions) -> Result<ExecResult, McpError> {
    let mut command = std::process::Command::new("apk");
    command.arg("update");

    if let Some(repository) = &refresh_options.repository {
        command.arg("--repository");
        command.arg(repository);
    }

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
        status: command.status.code().expect("exit code is expected"),
    })
}
