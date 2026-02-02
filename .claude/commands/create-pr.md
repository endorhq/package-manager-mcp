# Create a GitHub pull request

Create a GitHub Pull Request. The user also requests the following instructions for the creation of this Pull Request: "$ARGUMENTS". Ignore if empty.

Compare the changes between the branch in the user instructions or the current on and the `main` branch. Prioritize the branch provided in the user instructions if it exists.

Follow these rules to create the pull request content:

1. Take into account the user instructions if provided

2. When the user provides a branch name that it's different from the current one:

    - Do not run `git checkout` or running any command that changes the current user workspace. This is IMPORTANT
    - Use diff and log commands comparing the given branch and `main`
    - Do not run `git push`. Check if the given branch exists on the remote. If not, ask the user to push it manually and WAIT for his/her confirmation before continuing. This is IMPORTANT
3. Use a clear and concise title that summarizes the most relevant change in the pull request. The title follows the conventional commit format we use in this project.

    <good-example>
    feat: implement APT backend for Debian/Debian-derivative support

    fix: handle missing repository index in APK search
    </good-example>
    <bad-example>
    Refactor the backend modules into a separate crate

    Fix an issue
    </bad-example>

4. The pull request body must be informative, clear, and concise. Use a neutral language that explain the changes so users can quickly identify the changes in the pull request.

5. Do not include any checklist at the end

6. Do not include any "Created by Claude" comment at the end

7. Use GitHub markdown format in the body. For example, use the code blocks to show pieces of code, inline code blocks to highlight methods in paragraphs and list items, mermaid diagrams for complex pull requests, and tables when required.

8. Follow this template:

    <template>
    Brief summary for the changes in the pull request. 2 paragraphs max.

    # Changes

    Enumerate the major and most important changes in the pull request. Keep the list clear, short and concise.

    # Notes

    An optional section to indicate any other relevant information, major architectural change, future work, other changes that were not part of the task by were implemented in this pull request. You can also include a mermaid diagram if the changes are complex and a diagram might help to understand them.
    </template>

    <good-example>
    Add APT backend implementing the `PackageManager` trait for Debian/Debian-derivative systems, enabling package installation, search, and repository management via `apt-get` and `apt-cache`.

    ## Changes

    - Added `Apt` struct implementing the `PackageManager` trait
    - Implemented `install_package` and `install_package_with_version` using `apt-get install` with `DEBIAN_FRONTEND=noninteractive`
    - Implemented `search_package` using `apt-cache search` and version lookup via `apt-cache madison`
    - Added Debian OS detection via `/etc/debian_version` in `main.rs`

    ## Notes

    The APT backend follows the same pattern established by the APK backend, using `std::process::Command` for all operations and returning structured `ExecResult` values.
    </good-example>

    <good-example>
    Introduce the `install_package_with_version` MCP tool across all backends, allowing agents to pin specific package versions during installation.

    ## Changes
    - Added `InstallVersionOptions` type with `name` and `version` fields
    - Added `install_package_with_version` method to the `PackageManager` trait
    - Implemented version-specific installation for APK (multi-repository search) and APT (`apt-cache madison` lookup)
    - Registered the new tool in `PackageManagerHandler` with appropriate JSON schema

    ## Notes
    For APK, the implementation searches across multiple Alpine repositories (edge, v3.22, v3.21) to find the requested version. This is necessary because a specific version may only be available in certain repositories.

    ```mermaid
    flowchart TD
    Request([install_package_with_version]) --> Backend{Which backend?}
    Backend -->|APK| SearchRepos[Search across Alpine repositories]
    SearchRepos --> Found{Version found?}
    Found -->|Yes| InstallAPK[apk add pkg=version --repository repo_url]
    Found -->|No| ErrorAPK[Return error with searched repos]
    Backend -->|APT| Madison[apt-cache madison pkg]
    Madison --> VersionMatch{Version in output?}
    VersionMatch -->|Yes| InstallAPT[apt-get install pkg=version]
    VersionMatch -->|No| ErrorAPT[Return error with available versions]
    ```
    </good-example>

    <bad-example>
    Improve the command output handling and add a new Docker container tag to avoid collisions.
    </bad-example>

    <bad-example>
    ## Summary

    Replaces all `unwrap()` and `expect()` calls with proper `Result` error handling across the codebase.

    ## Changes

    - **src/backend/apk.rs**: Replaced `unwrap()` calls with `?` operator for command execution
    - **src/backend/apt.rs**: Replaced `expect()` calls with proper error propagation
    - **src/backend/mod.rs**: Updated handler methods to return `McpError` instead of panicking

    ## Benefits

    - **Reliability**: Server no longer panics on unexpected errors
    - **Observability**: Errors are properly propagated to MCP clients
    - **Consistency**: Uniform error handling pattern across all backends

    ## Test Plan

    - [x] Verified all MCP tools work correctly with the new error handling
    - [x] Tested error cases return proper MCP error responses

    Generated with Claude Code
    </bad-example>

9. Use the `gh` CLI to create the pull request
