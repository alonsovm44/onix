# Onix 💎

[![Language](https://img.shields.io/badge/language-rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-alpha-red.svg)](#)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](#)

Onix is not a package manager, not a registry, and it does not execute remote installation scripts (no `curl | sh`). No new ecosystem required.

# One-liner

> Onix installs CLI tools through a verifiable, declarative protocol that lets you preview, approve, and control every system change before it happens.

## The Problem

Installing standalone CLI tools today usually forces a tradeoff between trust and convenience:

1. **The Risk:**  
   Running `curl | sh`-style installers that execute remote code with no structured review or guarantees.

2. **The Friction:**  
   Using OS-specific or language-specific package managers (Homebrew, APT, Winget, NPM, Cargo), which often introduce:
   - ecosystem lock-in  
   - dependency overhead  
   - inconsistent installation behavior across platforms  

In both cases, installation is either opaque or unnecessarily complex.

## The Onix Solution

Onix introduces a **declarative installation protocol for CLI binaries**.

Instead of executing scripts or relying on package ecosystems, Onix:
- resolves a published release artifact
- verifies integrity (e.g. checksums)
- shows a transparent installation plan
- requires explicit user approval before making system changes

This combines the simplicity of a single command with the safety of a structured, inspectable installation flow.

## Design Principles

- **No hidden execution:** Onix never runs remote code. No curl/irm hacks.
- **Explicit changes:** Every system modification is shown before execution.
- **Artifact-first:** Software is distributed as verifiable binaries, not scripts.
- **User-controlled installs:** Nothing is installed without explicit approval.

### Key Features
- **🛡️ Trust-First:** Every installation requires explicit user consent via a TUI permission prompt.
- **✅ Integrity:** Mandatory SHA256 checksum verification for every artifact.
- **📦 Declarative:** Authors define a simple YAML manifest; Onix handles the system-level "plumbing."
- **🌍 Cross-Platform:** One protocol for Linux, macOS, and Windows.
- **🧹 Clean Uninstalls:** Since Onix tracks exactly what it touches, it can revert changes completely.

## How it Works

### 1. The Manifest (`install.onix`)
Authors publish a simple YAML file alongside their releases. This file tells Onix exactly what to download and what permissions are needed.

```yaml
schema: "1.0.0"
app: "my-awesome-tool"
version: "1.2.3"
install-on:
  - os: "linux"
    arch: "amd64"
    url: "https://example.com/bin/linux-amd64.tar.gz"
    sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"

installation:
  file-type: "binary"
  target-dir: "~/.local/bin"
  bin-name: "atool"

permissions:
  - "write:~/.local/bin"
  - "env:PATH"
```

### 2. The Safe Install
When a user runs `onix install user@repo`, Onix:
1. Fetches and validates the manifest.
2. Matches the architecture and OS.
3. **Displays a TUI prompt** showing exactly which files will be written and which environment variables will change.
4. Downloads, verifies the SHA256, and installs only after user confirmation.

## Comparison

| Feature | `curl | sh` | Package Managers | Onix |
| :--- | :---: | :---: | :---: |
| **Ease of Use** | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
| **Security** | ❌ | ✅ | ✅ |
| **Transparency** | ❌ | ❌ | ✅ |
| **Cross-Platform** | ⚠️ | ❌ | ✅ |
| **No Bloat** | ✅ | ❌ | ✅ |

## Installation

*Coming Soon: Onix is currently in early alpha.*

```bash
# Proposed bootstrap method
onix self-install
```

## FAQ

**Why not just use Winget or Chocolatey?**
Onix is cross-platform. A developer writes one `.onix` file and it works everywhere. It also provides a decentralized "bring-your-own-host" model.

**Is this a replacement for Cargo or NPM?**
No. Cargo and NPM manage libraries and build dependencies. Onix is for distributing the *final result*: the standalone binary.

**Does it require Admin/Sudo?**
Only if you try to install to a system directory. Onix encourages "User-land" installs (like `~/.local/bin`) to keep your system clean.

## License
This project is licensed under the MIT License.

---
*Onix: Install with confidence, not just hope.*

can we add "onix publish --v x.x.x " so it automatically stages, commits, pushes, makes the tag with the x.x.x version, and pushes the tag"?  