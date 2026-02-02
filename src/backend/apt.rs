use rmcp::ErrorData as McpError;

use super::{ExecResult, InstallOptions, InstallVersionOptions, PackageManager, SearchOptions};

/// Debian/Debian-derivative APT package manager backend
#[derive(Clone)]
pub struct Apt;

impl Apt {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Apt {
    fn default() -> Self {
        Self::new()
    }
}

impl PackageManager for Apt {
    fn name(&self) -> &'static str {
        "APT"
    }

    fn os_name(&self) -> &'static str {
        "Debian/Debian-derivative"
    }

    fn install_package(&self, options: &InstallOptions) -> Result<ExecResult, McpError> {
        let mut command = std::process::Command::new("apt-get");
        command.env("DEBIAN_FRONTEND", "noninteractive");
        command.arg("install");
        command.arg("-y");

        if let Some(repository) = &options.repository {
            command.arg("-o");
            command.arg(format!("Dir::Etc::sourcelist={repository}"));
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
                    "Invalid package name '{}': only alphanumeric characters, dots, hyphens, underscores, plus signs, and colons are allowed",
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
                    "Invalid version string '{}': only alphanumeric characters, dots, hyphens, underscores, plus signs, colons, and tildes are allowed",
                    options.version
                ),
                Some(serde_json::json!({
                    "version": options.version,
                    "error_type": "validation_error"
                })),
            ));
        }

        // First, check available versions using apt-cache madison
        let madison_output = std::process::Command::new("apt-cache")
            .arg("madison")
            .arg(&options.package)
            .output()
            .map_err(|err| {
                McpError::internal_error(
                    format!(
                        "there was an error checking versions for package {}: {}",
                        options.package, err
                    ),
                    None,
                )
            })?;

        let mut found_versions: Vec<String> = Vec::new();
        let mut version_found = false;

        if madison_output.status.success() {
            let stdout = String::from_utf8_lossy(&madison_output.stdout);
            for line in stdout.lines() {
                // apt-cache madison output format: package | version | source
                let parts: Vec<&str> = line.split('|').collect();
                if parts.len() >= 2 {
                    let version = parts[1].trim().to_string();
                    if version == options.version {
                        version_found = true;
                    }
                    if !found_versions.contains(&version) {
                        found_versions.push(version);
                    }
                }
            }
        }

        // If exact version match found (or we couldn't verify), try to install it
        if version_found || found_versions.is_empty() {
            let mut command = std::process::Command::new("apt-get");
            command.env("DEBIAN_FRONTEND", "noninteractive");
            command.arg("install");
            command.arg("-y");
            command.arg(format!("{}={}", options.package, options.version));

            let output = command.output().map_err(|err| {
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
        // Note: APT doesn't support custom repository for search, uses system sources
        let output = std::process::Command::new("apt-cache")
            .arg("search")
            .arg(&options.query)
            .output()
            .map_err(|err| {
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
        let output = std::process::Command::new("apt")
            .arg("list")
            .arg("--installed")
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
        let output = std::process::Command::new("apt-get")
            .env("DEBIAN_FRONTEND", "noninteractive")
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
    // Allow alphanumeric, dots, hyphens, underscores, plus signs, colons, and tildes
    // (colons are common in Debian package names like "package:amd64", tildes in versions like "1.0~beta")
    input.chars().all(|c| {
        c.is_alphanumeric() || c == '.' || c == '-' || c == '_' || c == '+' || c == ':' || c == '~'
    })
}
