use rmcp::ErrorData as McpError;

use super::{ExecResult, InstallOptions, InstallVersionOptions, PackageManager, SearchOptions};

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

/// Alpine Linux APK package manager backend
#[derive(Clone)]
pub struct Apk;

impl Apk {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Apk {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageManager for Apk {
    fn name(&self) -> &'static str {
        "APK"
    }

    fn os_name(&self) -> &'static str {
        "Alpine Linux"
    }

    fn install_package(&self, options: &InstallOptions) -> Result<ExecResult, McpError> {
        let mut command = std::process::Command::new("apk");
        command.arg("add");

        if let Some(repository) = &options.repository {
            command.arg("--repository");
            command.arg(repository);
        }

        command.arg(&options.package);

        let output = command.output().map_err(|err| {
            McpError::internal_error(
                format!(
                    "there was an error installing package {}: {}",
                    &options.package, err
                ),
                None,
            )
        })?;

        Ok(ExecResult {
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
        })
    }

    fn install_package_with_version(
        &self,
        options: &InstallVersionOptions,
    ) -> Result<ExecResult, McpError> {
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

        let search_result = self.search_package(&search_options)?;

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

    fn search_package(&self, options: &SearchOptions) -> Result<ExecResult, McpError> {
        let mut command = std::process::Command::new("apk");
        command.arg("--no-cache");

        // Add repositories: use provided repository or search all
        if let Some(repository) = &options.repository {
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
        command.arg(&options.query);

        let output = command.output().map_err(|err| {
            McpError::internal_error(
                format!(
                    "there was an error searching for packages with query {}: {}",
                    &options.query, err
                ),
                None,
            )
        })?;

        Ok(ExecResult {
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
        })
    }

    fn list_installed_packages(&self) -> Result<ExecResult, McpError> {
        let output = std::process::Command::new("apk")
            .arg("list")
            .arg("-I")
            .output()
            .map_err(|err| {
                McpError::internal_error(
                    format!("there was an error listing installed packages: {err}"),
                    None,
                )
            })?;

        Ok(ExecResult {
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
        })
    }

    fn refresh_repositories(&self) -> Result<ExecResult, McpError> {
        let output = std::process::Command::new("apk")
            .arg("update")
            .output()
            .map_err(|err| {
                McpError::internal_error(
                    format!("there was an error refreshing repositories: {err}"),
                    None,
                )
            })?;

        Ok(ExecResult {
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
        })
    }
}

fn validate_package_version_input(input: &str) -> bool {
    // Allow alphanumeric, dots, hyphens, underscores, and plus signs (common in version strings)
    input
        .chars()
        .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == '+')
}
