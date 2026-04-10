# Onix Workflow: From Author's Compile to User's Install
This workflow assumes the Onix client application is already installed on the user's machine.

## Phase 1: The Author's Workflow (Preparing the Tool for Onix)
This phase focuses on the developer who has created a CLI tool and wants to distribute it using Onix.

- Compile Binaries for Target Platforms:

1. The author compiles their CLI tool for all desired operating systems and architectures (e.g., Linux AMD64, Windows AMD64, macOS ARM64).
Onix Relevance: Onix is designed for pre-compiled binaries, making this the first crucial step.
Calculate SHA256 Checksums:

For each compiled binary, the author calculates its SHA256 cryptographic hash. This is vital for integrity verification.
Onix Relevance: These checksums will be included in the .onix manifest, allowing the Onix client to verify the downloaded binary's integrity.
Create the .onix Manifest (YAML):

2. The author creates a declarative YAML file (e.g., install.onix) that describes the tool and its installation process. This manifest includes:
- schema version.
- app name and version.
- install-on entries for each platform, specifying the os, arch, url to the binary, and its sha256 checksum.
- installation details: file-type (e.g., binary), target-dir (e.g., ~/.local/bin), and bin-name.
- permissions: A list of explicit system actions the installation will take (e.g., write:~/.local/bin, env:PATH).
- An optional message to display after successful installation.
Onix Relevance: This manifest is the core of Onix's declarative approach, providing transparency and verifiability.
Host Binaries and the .onix Manifest:

3. The author uploads the compiled binaries to a reliable hosting service (e.g., GitHub Releases, a CDN, their own server).

4. The author also hosts the .onix manifest file, typically alongside the binaries or in a well-known location.
Onix Relevance: Onix fetches both the manifest and the binaries from these URLs. Using HTTPS for hosting is a critical security best practice.
Distribute the onix install Command:

5. The author provides users with a simple command to install their tool, pointing to the hosted .onix manifest.
Example: onix install https://github.com/mycli/mycli/releases/latest/download/install.onix
Onix Relevance: This is the "one-liner" that replaces curl | sh, offering convenience with built-in trust.

## Phase 2: The User's Workflow (Installing the Tool with Onix)
This phase describes what happens when a user executes the onix install command.

1. User Initiates Installation:

The user opens their terminal and runs the onix install <URL_TO_ONIX_MANIFEST> command provided by the author.
Onix Fetches the Manifest:

2. The Onix client downloads the .onix manifest file from the specified URL (e.g., https://github.com/mycli/mycli/releases/latest/download/install.onix).
Onix Relevance: This is the first step in understanding what the user is about to install.
Onix Validates the Manifest:

3. Onix parses the downloaded YAML manifest and validates it against its internal schema to ensure it's well-formed and adheres to the Onix specification.
Onix Relevance: Prevents malformed or malicious manifests from proceeding.
Onix Selects the Correct Binary Source:

4. Based on the user's operating system and architecture, Onix identifies the appropriate url and sha256 from the install-on section of the manifest.
Onix Relevance: Ensures cross-platform compatibility and selects the right artifact for the user's system.
Onix Presents the Permission Prompt (TUI):

5. Onix displays a clear, interactive Text User Interface (TUI) to the user, summarizing the installation plan. This prompt includes:
- The tool's app name and version.
- The URL from which the binary will be downloaded.
- The target installation directory (target-dir) and binary name (bin-name).
- A list of explicit permissions the installation requires (e.g., "Write to ~/.local/bin", "Modify PATH environment variable").
- A clear "Proceed? [Y/n]" prompt.
Onix Relevance: This is the core "trust primitive" and differentiator. It provides informed consent, allowing the user to review and approve all actions before any changes are made to their system.
User Confirms Installation:

6. The user reviews the proposed actions and permissions. If satisfied, they confirm the installation (e.g., by typing y and pressing Enter). If not, they can cancel.

7. Onix Downloads the Binary:

8. Upon user confirmation, Onix downloads the selected binary from its specified url.
Onix Relevance: Direct download, no intermediate scripts.
Onix Verifies SHA256 Checksum:

9. After downloading, Onix calculates the SHA256 checksum of the downloaded binary and compares it against the sha256 value specified in the manifest.
Onix Relevance: Ensures the downloaded file has not been tampered with during transit and matches what the author intended. If checksums don't match, the installation is aborted.
Onix Installs the Binary:

10. Onix moves the downloaded binary to the specified target-dir.
11. It renames the binary to bin-name if necessary.
12. It sets appropriate file permissions (e.g., chmod +x on Unix-like systems).
Onix Relevance: Automates the "plumbing" of manual installation.
Onix Updates Environment Variables (if specified):

13. If the manifest includes permissions related to environment variables (e.g., env:PATH), Onix modifies the user's shell configuration file (e.g., .bashrc, .zshrc) or system-level environment variables (on Windows) to include the new binary's path.
Onix Relevance: Ensures the installed tool is immediately accessible from the command line without manual configuration.
Onix Provides Success Message:

14. Onix displays a success message, potentially including the message from the manifest (e.g., "Run mycli --help to get started").
Onix Relevance: Confirms successful installation and guides the user on next steps.
This step-by-step process clearly delineates the responsibilities and benefits for both developers and users, showcasing how Onix bridges the gap between convenience and trust.

# 🧾 One-line explanation

Onix automates the manual binary install process—adding verification, consistency, and explicit permissions—without executing remote scripts.