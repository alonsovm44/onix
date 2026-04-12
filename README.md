# Onix 💎

**Onix** is a developer-centric binary distribution and release automation tool. It streamlines the entire lifecycle of CLI applications—from initial CI/CD scaffolding to automated multi-platform releases and seamless client-side installations.

## The one liner
> Opinionated binary distribution pipeline for GitHub-native CLI and standalone executable apps

## ✨ Features

*   **Zero-Config Scaffolding**: `onix init` instantly sets up your project with a `.onix/config.yaml` and a production-ready GitHub Actions workflow.
*   **TUI-Powered Publishing**: The `onix publish` command automates your Git workflow (stage, commit, tag, push) and provides a real-time Terminal UI to track CI build progress.
*   **Automated Artifact Verification**: Automatically fetches release assets, verifies SHA-256 integrity, and generates an `install.onix` manifest for the consumer.
*   **Smart Installation**: `onix install` handles binary deployment and maintains a `deprecated` archive of previous versions to ensure safe rollbacks and updates.
*   **Multi-Platform by Default**: Built-in support for Linux (x86_64), macOS (ARM64/x86_64), and Windows (x86_64).

## 🚀 Getting Started

### Installation

Currently, Onix is built using Rust. You can compile it from source:

```bash
cargo build --release
```

### Initializing a Project

Run the following command in the root of your project:

```bash
onix init
```

This will create:
1.  `.onix/config.yaml`: Central configuration for your app metadata and build targets.
2.  `.github/workflows/release.yml`: A pre-configured GitHub Action matrix build.

## 📦 Usage

### Publishing a Release

To release a new version, simply run:

```bash
onix publish [VERSION]
```

**What happens under the hood:**
1.  Updates your version in `.onix/config.yaml`.
2.  Stages all changes and creates a release commit.
3.  Pushes a new Git tag (e.g., `v0.1.7`).
4.  Opens a **Terminal UI** to poll GitHub Actions status, showing real-time progress of your build matrix.
5.  Calculates checksums for all platform artifacts.
6.  Generates and uploads an `install.onix` manifest to the GitHub Release.

### Installing a Package

Consumers can install your tool by pointing to the repository:

```bash
onix install <owner>/<repo>
```

Onix will detect existing versions, move them to the `deprecated` folder with a timestamp, and deploy the fresh binary to your toolset root.

## ⚙️ Configuration (`config.yaml`)

The `.onix/config.yaml` file defines how your application is built and distributed:

```yaml
app:
  name: my-app
  version: 0.1.0
build:
  entry: src/main.rs
  command: cargo build --release
  output_name: my-app
targets:
  - { os: linux, arch: x86_64 }
  - { os: macos, arch: arm64 }
  - { os: windows, arch: x86_64 }
install:
  file_type: binary
  bin_name: my-app
```

## 🔐 Security

Onix handles GitHub authentication via a `GITHUB_TOKEN`.
*   It looks for a `GITHUB_TOKEN` environment variable.
*   Alternatively, it stores a local token in `.onix/token.key` (automatically added to your `.gitignore`).

## 🛠️ Requirements

*   **Git**: Must be installed and configured in your shell.
*   **GitHub Repository**: The `origin` remote must point to a GitHub URL.

---
*Built with Rust and ❤️ for the developer community.*