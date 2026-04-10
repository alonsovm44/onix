# Onix 1.0.0 MVP Roadmap

## Phase 1: Foundation & The Validation Engine (The "Logic")
Focus on the plumbing required to handle the .onix manifest safely.

[x ] CLI Scaffolding: Implement clap for subcommands (install, inspect, validate).
[x ] Manifest Parser: Robust YAML parsing using serde_yaml based on our models.rs.
[ x] Hashing Utility: Implementation of SHA256 verification logic to ensure artifact integrity.
[x ] onix validate: Enable developers to check their YAML schema locally against the spec.

## Phase 2: The Trust Primitive (The "Interface")
Build the TUI that differentiates Onix from curl | sh.

[ ] TUI Design: Create the ratatui interface for the permission prompt.
[ ] onix inspect: Implement the dry-run mode that fetches the manifest and prints the intended actions without touching the disk.
[ ] Interactive Confirmation: Logic to halt execution until the user explicitly accepts the permissions shown in the TUI.

## Phase 3: System Integration (The "Execution")
Actually move bytes and modify the system environment.

[ ] Download Engine: Secure file downloading with progress bars.
[ ] Atomic Installation: Logic to move binaries to target-dir and set execution bits (chmod +x).
[ ] Environment Manager: Cross-platform PATH modification (updating .zshrc/.bashrc on Unix or Registry on Windows).
[ ] --yes Flag: Skip the TUI for CI/CD and automation scripts.

## Phase 4: Developer Experience (The "Distribution")
Make it easy for authors to adopt Onix.

[ ] onix init: Interactive wizard to detect project types (Rust, C++, Go) and generate the .onix/config.yaml.
[ ] Template Generation: Automatic creation of the GitHub Actions workflow and the install.onix template.
[ ] onix publish: Command to generate the build matrix and calculate initial hashes.

## Phase 5: Persistence & Cleanup
[ ] Installation Registry: A local JSON/SQLite store to track what Onix has installed (preparing for onix uninstall).
[ ] Self-Install: The bootstrap command to get Onix itself onto a fresh machine.