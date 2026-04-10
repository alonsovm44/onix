# PIPELINE v1.0.1
**Overall Plan**
The implementation will revolve around three main commands:

`onix init`: Initializes a new Onix project by creating a default .onix/config.yaml file.
`onix publish`: Automates the release process. It creates a Git tag, pushes it to trigger CI, polls GitHub Actions for completion, fetches the built binaries, computes their SHA256 checksums, generates the install.onix manifest, uploads it to the GitHub Release, and finally publishes the release.
`onix install user@repo`: Handles the installation of an application. It resolves the repository, fetches the latest GitHub Release, downloads the generated install.onix from that release, presents a preview to the user, and then performs the installation of the appropriate binary.

## 1. onix init Command
Goal: Create a default .onix/config.yaml if one doesn't exist.

Implementation Details:

Check for the existence of .{onix}/config.yaml in the current directory or a parent directory.
If not found, create the .onix directory and a config.yaml file within it.
Populate config.yaml with sensible defaults, similar to the example provided in the context. This allows developers to get started quickly and then customize as needed.

## 2. onix publish Command
Goal: Automate the entire release process, from tagging to publishing the final install.onix manifest.

Implementation Details:

Load Configuration: Read .onix/config.yaml to get application name, version, build output name, and target platforms.
Determine Repository Information: Extract the GitHub owner and repository name from the Git remote URL.
Create Git Tag: Create a new Git tag (e.g., v1.0.1) based on the app.version from config.yaml.
Push Tag: Push the newly created tag to the remote repository. This action is expected to trigger a GitHub Actions workflow that builds the binaries for all specified targets and uploads them as assets to a draft GitHub Release.
Poll GitHub Actions: Implement a polling mechanism to monitor the status of the GitHub Actions workflow triggered by the tag push. This involves:
Making GitHub API calls to list workflow runs.
Filtering for the workflow run associated with the pushed tag.
Continuously checking its status until it completes (success or failure).
Implementing exponential backoff to avoid hitting API rate limits.
Handling timeouts for stalled builds.
Fetch Release Artifacts: Once the GitHub Actions workflow successfully completes, retrieve the IDs of the draft release and download the built binary artifacts from it. These artifacts are the cross-compiled binaries for each target.
Compute SHA256 Checksums: For each downloaded binary artifact, calculate its SHA256 checksum.
Generate install.onix Manifest: Using the loaded config.yaml, the repository information, the tag, and the computed SHA256 checksums, dynamically generate the install.onix YAML content. This is a critical step that replaces the manual manifest maintenance.
Upload install.onix: Upload the newly generated install.onix content as an asset to the existing draft GitHub Release.
Publish Release: Change the status of the draft GitHub Release to a published release.

## 3. onix install user@repo Command
Goal: Provide a verified and previewed installation experience for users.

Implementation Details:

Parse Repository Slug: Extract the owner and repository name from the user@repo string.
Fetch Latest Release: Use the GitHub API to find the latest published release for the specified repository.
Download install.onix: Locate and download the install.onix asset from the fetched release. This ensures the manifest is versioned and immutable, directly corresponding to the binaries in that release.
Parse install.onix: Deserialize the downloaded install.onix content into an internal data structure.
Show TUI (Preview): Present the installation details to the user (e.g., application name, version, target directory, binaries available for their system). This allows the user to review before proceeding.
Identify Current System: Determine the user's operating system and architecture.
Download and Verify Binary: Find the appropriate binary URL from the install.onix for the user's system, download it, and verify its integrity using the provided SHA256 checksum.
Install Binary: Place the downloaded binary in the specified target-dir, rename it to bin-name, and set appropriate file permissions.
Handle Permissions/Environment: If install.onix specifies environment variable modifications (e.g., adding target-dir to PATH), guide the user or attempt to modify it (with appropriate warnings and user consent).