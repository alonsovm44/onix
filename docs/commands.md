
# commands to be implemented for MVP v1.0.0

```bash
onix install <url>
# Fetches manifest
# Shows plan (TUI)
# Asks for confirmation
# Downloads binary
# Verifies checksum
# Installs binary
# Updates PATH

onix inspect <url> # this is the dry run command
# Fetches manifest
# Parse + validate
# Show install plan
# does NOT install anything

onix install <url> --yes
#skips confirmation prompt
# required for CI/CD, automation

onix validate <manifest.onix>
# validates the .onix yaml schema localy to catch errors

onix --help
# shows help

onix --version | -v 
# shows version installed

onix install <url> --versbose
#verbose install mode
```
# Publish

```bash
onix publish 
# produces compilation matrix for project
```
For `publish` output:

# Init 

```bash
onix init
# prepares the repo for a onix project

```

What happens:

1. detects project type (C++, Go, Rust, etc.)
2. asks minimal questions:
- app name
- version
- entry file

generates:
1. .onix/config.yaml (optional internal config)
2. .github/workflows/onix.yml

## config file
this generates on .onix/config.yaml
```yaml
app:
  name: mycli
  version: 1.0.0

build:
  entry: app.cpp
  command: g++ app.cpp -O2 -o mycli

output:
  name: mycli
  dir: dist

targets:
  - os: linux
    arch: amd64

  - os: linux
    arch: arm64

  - os: macos
    arch: arm64

  - os: windows
    arch: amd64

release:
  provider: github
  tag_prefix: v
  generate: true

onix:
  generate_manifest: true
  manifest_name: install.onix

install:
  bin_name: mycli
  target_dir: ~/.local/bin

permissions:
  - type: filesystem
    action: write
    path: ~/.local/bin

  - type: environment
    action: modify
    variable: PATH

message: Run `mycli --help` to get started
```

👉 No editing required

For `init` output:
```cmd
Detected project: C++ (g++)

CLI name: mycli
Version: 1.0.0

Supported platforms:
  [x] Linux amd64
  [x] Linux arm64
  [x] macOS arm64
  [x] Windows amd64

Install directory: ~/.local/bin

Generate workflow + install.onix? [Y/n]
```
- 👉 User hits Y

## What onix init generates

### 1. GitHub Actions workflow
.github/workflows/onix.yml

This contains:

- matrix builds (Linux/macOS/Windows)
- compile commands
- artifact upload
- SHA256 generation
- release upload (optional in MVP)

This generates with `init` on .github/workflows/onix.yml
```yml
name: Onix Build & Release

on:
  push:
    tags:
      - "v*"

jobs:
  build:
    runs-on: ${{ matrix.os }}

    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            name: linux-amd64
            build_cmd: g++ app.cpp -O2 -o mycli
            out: mycli

          - os: ubuntu-latest
            name: linux-arm64
            build_cmd: aarch64-linux-gnu-g++ app.cpp -O2 -o mycli
            out: mycli

          - os: macos-latest
            name: macos-arm64
            build_cmd: g++ app.cpp -O2 -o mycli
            out: mycli

          - os: windows-latest
            name: windows-amd64
            build_cmd: g++ app.cpp -O2 -o mycli.exe
            out: mycli.exe

    steps:
      - name: Checkout repo
        uses: actions/checkout@v4

      - name: Install dependencies (Linux cross tools)
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt update
          sudo apt install -y g++ mingw-w64 gcc-aarch64-linux-gnu

      - name: Build binary
        run: ${{ matrix.build_cmd }}

      - name: Rename artifact
        run: |
          mkdir -p dist
          mv ${{ matrix.out }} dist/${{ matrix.name }}

      - name: Generate SHA256
        run: |
          cd dist
          sha256sum * > checksums.txt || certutil -hashfile * SHA256 > checksums.txt

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.name }}
          path: dist/

  release:
    needs: build
    runs-on: ubuntu-latest

    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: dist

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v2
        with:
          files: dist/**/*
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}

```

# install.onix template

install.onix

With placeholders or pre-filled values:
```yaml
schema: 1.0.0

app: mycli
version: 1.0.0

install-on:
  - os: linux
    arch: amd64
    url: https://github.com/USER/REPO/releases/download/v1.0.0/mycli-linux-amd64
    sha256: <filled by CI>

installation:
  type: binary
  target-dir: ~/.local/bin
  bin-name: mycli

permissions:
  - type: filesystem
    action: write
    path: ~/.local/bin

  - type: environment
    action: modify
    variable: PATH

message: Run `mycli --help` to get started
```
👉 CI later fills in hashes automatically.

👉 User does NOT write YAML manually.