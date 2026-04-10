# 🚀 Onix MVP (Version 0.1)
## 🧠 MVP goal (keep this extremely tight)

>Let a user install a CLI tool from a URL with:

- visible steps
- explicit permissions
- deterministic downloads
- no shell execution of remote code

That’s it.

## 🧩 1. MVP scope (what Onix DOES)
Core command
```bash
onix install https://example.com/install.onix
```
Flow:
1. Fetch .onix file
2. Validate schema (basic)
3. Show install plan (TUI or simple CLI output)
4. Ask for confirmation
5. Download correct binary
6. Verify SHA256
7. Place binary in target dir
8. Update PATH (optional, explicit)

## 📄 2. Minimal .onix schema (MVP version)

Cut everything non-essential.
```json
{
  "schema": "onix/v1",
  "name": "mycli",
  "version": "1.0.0",

  "source": {
    "os": "linux",
    "arch": "amd64",
    "url": "https://example.com/mycli",
    "sha256": "abc123"
  },

  "install": {
    "type": "binary",
    "target": "~/.local/bin",
    "bin": "mycli"
  },

  "permissions": [
    "write:~/.local/bin",
    "env:PATH"
  ],

  "message": "Run `mycli --help` to get started"
}
```
Key simplifications vs v1 spec:
❌ no multiple sources
❌ no archives
❌ no hooks
❌ no dependencies
❌ no signing system (yet)

## 🖥 3. MVP UX (this is your differentiator)
Install preview (critical)
```
Onix will install:

  mycli 1.0.0

Actions:
  - Download: https://example.com/mycli
  - Verify: sha256 match required
  - Install: ~/.local/bin/mycli
  - Modify: PATH

Permissions:
  - write ~/.local/bin
  - modify PATH

Proceed? [y/N]
```
👉 This is your entire product in one screen

Success output
✓ Downloaded
✓ Verified
✓ Installed mycli

Run:
  mycli --help
🔐 4. Trust model (MVP-level)

Keep it simple:

Required:
HTTPS only
SHA256 required
explicit user confirmation
Optional (NOT v1):
signatures
SBOM
key management

👉 Don’t overbuild trust infrastructure yet—prove demand first.

⚙️ 5. Implementation components
1. CLI binary (onix)
written in Rust / Go / C++ (your choice)
responsibilities:
fetch JSON
parse
validate
execute install steps
2. Schema validator
strict JSON schema check
reject unknown fields (important for trust)
3. Downloader
fetch binary
compute SHA256
compare
4. Installer
copy to target directory
chmod +x
optional PATH update
5. PATH updater (careful scope)

MVP approach:

only support:
.bashrc
.zshrc

Append:

export PATH="$HOME/.local/bin:$PATH"

## What Onix Is and What It Does

Onix is envisioned as a universal, lightweight, trust-first installer for standalone CLI binaries. Its primary goal is to standardize and secure the process of distributing and installing native command-line tools, especially those written in languages like C/C++.

- Here's what it does:

Solves the Trust Problem: It addresses the hesitation users have when running arbitrary curl | sh scripts from the internet. Onix aims to make installations transparent and auditable by explicitly declaring permissions and verifying artifacts.

Solves the Distribution Problem: For developers, it simplifies the process of distributing CLI tools without needing to maintain multiple platform-specific package manager configurations (like Homebrew, apt, winget). For users, it provides a simple, direct installation method.

Solves the Fragmentation Problem: It aims to provide a consistent, cross-platform installation experience, replacing the current situation where every tool might have different, often complex, installation instructions.

Provides a Declarative Installation: Developers publish a .onix manifest (like the YAML example you provided) that declaratively describes what an installation entails: where to download the binary, its checksum, where it should be placed, and what system permissions it requires.

Offers a Transparent User Experience: Before execution, the Onix installer app will verify signatures and checksums, and crucially, present a clear permission 
prompt (a TUI) to the user, summarizing exactly what changes will occur on their system. This allows users to make an informed decision before proceeding.

In essence, Onix wants to offer the convenience of a one-liner install (onix install ...) with the trust and structure typically associated with a package manager, but without the heavy overhead of becoming a full-fledged package manager itself.

## How Onix Differentiates from Package Managers like Cargo or npm
The core distinction lies in their scope and purpose:

- Scope:

- - Cargo (Rust) and npm (Node.js): These are language ecosystem package managers. They operate inside a specific programming language's ecosystem. Their primary function is to manage dependencies for projects written in that language, fetch libraries, compile code (in Cargo's case), and install tools within that ecosystem. They are deeply integrated with the language's build system and runtime.

- - Onix: Onix is designed for cross-language CLI distribution. It's not tied to any single programming language or ecosystem. It focuses specifically on installing standalone, pre-compiled binaries onto a user's system, regardless of the language they were written in.

- Problem Solved:

- - Cargo/npm: Primarily solve the problem of managing project dependencies and distributing libraries/tools within their respective language ecosystems.

- - Onix: Primarily solves the "trust problem" and "distribution problem" for standalone native CLI tools that often fall outside the neat boundaries of a single language ecosystem or require complex OS-level packaging. It's competing with the curl | sh pattern, not with cargo install or npm install -g.

- Mental Model:

- - Cargo/npm: "Get me a Rust/JavaScript package (and its dependencies) for my project or globally."

- - Onix: "Install this verified tool onto my machine safely, declaring exactly what changes will happen."

- "Package" Definition:

- - Cargo/npm: A "package" often refers to source code, libraries, and metadata that need to be built or interpreted.
 
- - Onix: A "package" (or rather, an "artifact") is typically a pre-compiled binary or an archive containing binaries, along with a declarative manifest (.onix file) that describes its installation.

The YAML schema provided (install.onix) perfectly illustrates this differentiation. It's not describing how to build a Rust crate or a Node.js module; instead, it's declaring:

App and version: The identity of the standalone tool.
install-on: Where to find the pre-built binary for specific OS/architecture combinations, along with its sha256 for integrity verification.
installation: How to place that binary (file-type: binary, target-dir, bin-name).
permissions: Explicitly stating what system-level actions the installation will take (e.g., write:~/.local/bin, env:PATH).

This declarative approach, focused on pre-built artifacts and explicit system interactions, is what sets Onix apart from language-specific package managers. It's a "safe install protocol" rather than a full-blown package manager that handles compilation, dependency resolution, and versioning of source code.