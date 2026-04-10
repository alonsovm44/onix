# Onix ❄️

[![Language](https://img.shields.io/badge/language-rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-alpha-red.svg)](#)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](#)


# One-liner

>Onix lets you preview, verify, and approve every system change before installing any CLI tool.

## The Problem
Installing a new standalone CLI tool today usually involves one of two extremes:
1. **The Risk:** Running `curl | sh` scripts with blind trust.
2. **The Friction:** Wrestling with OS-specific package managers (Homebrew, APT, Winget) or language-specific ones (NPM, Cargo) that often come with heavy dependency chains.

## The Onix Solution
Onix bridges the gap. It provides the convenience of a one-liner installation with the security of a declarative protocol. 

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
When a user runs `onix install <url>`, Onix:
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
