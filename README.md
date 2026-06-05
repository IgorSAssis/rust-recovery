# RustRecover

[![CI](https://github.com/IgorSAssis/rust-recovery/actions/workflows/ci.yml/badge.svg)](https://github.com/IgorSAssis/rust-recovery/actions/workflows/ci.yml)
![Platform](https://img.shields.io/badge/platform-linux-blue)
![Rust](https://img.shields.io/badge/rust-stable-orange)

RustRecover scans raw disk images and storage devices searching for recoverable files using two complementary strategies: **signature-based carving** (finds files by their byte patterns, works anywhere) and **FAT32 filesystem parsing** (reads deleted directory entries directly, preserving original filenames).

Available as both a **desktop GUI** and a **command-line tool**.

---

## Table of Contents

- [Features](#features)
- [Recovery Strategies](#recovery-strategies)
- [Supported File Types](#supported-file-types)
- [Architecture](#architecture)
- [Getting Started](#getting-started)
- [Usage](#usage)
  - [Desktop UI](#desktop-ui)
  - [CLI](#cli)
- [Project Structure](#project-structure)
- [CI/CD](#cicd)
- [Roadmap](#roadmap)

---

## Features

- **Two recovery strategies** — signature-based carving and FAT32 filesystem parsing
- **Device detection** — automatically lists available storage devices
- **Image preview** — visualize recovered JPEG and PNG files before exporting
- **Selective export** — choose individual files or export everything at once
- **Internationalization** — English and Brazilian Portuguese, auto-detected from system locale

---

## Recovery Strategies

RustRecover implements two strategies that complement each other:

| | Carver | FAT32 |
|---|---|---|
| **How it works** | Searches for known byte signatures (magic numbers) in the raw data | Reads the FAT32 directory table looking for entries marked as deleted (`0xE5`) |
| **Works on** | Any device or image format | FAT32 volumes only |
| **Preserves filenames** | No — files are named by offset | Yes — original name recovered from directory entry |
| **Precision** | Good — depends on signature quality | High — knows exact file size and location |
| **Best for** | Severely damaged filesystems, non-FAT32 devices | USB drives, SD cards, cameras formatted as FAT32 |

Both strategies can be used on the same source. They recover different sets of files and are not mutually exclusive.

---

## Supported File Types

| Format | Extension | Strategy |
|---|---|---|
| JPEG | `.jpg`, `.jpeg` | Carver, FAT32 |
| PNG | `.png` | Carver, FAT32 |
| PDF | `.pdf` | Carver, FAT32 |
| ZIP / DOCX / XLSX | `.zip` | Carver, FAT32 |

> The FAT32 strategy recovers any file type whose directory entry is still intact, not just the formats listed above.

---

## Architecture

RustRecover is organized as a Cargo workspace where each crate has a single responsibility:

```
rust-recovery/
├── apps/
│   ├── cli/              # Command-line interface (clap)
│   └── ui/               # Desktop GUI (iced 0.14)
└── crates/
    ├── device_detector/  # Lists storage devices via /sys/block (Linux)
    ├── file_carver/      # Byte signature definitions, streaming scanner, extractor
    └── recovery_engine/  # Orchestrates strategies; owns RecoveryStrategy trait
```

The dependency flow is strictly one-directional — `apps` depend on `crates`, never the reverse:

```
cli / ui
    └── recovery_engine
            ├── file_carver      (Carver strategy)
            └── [fat32 parser]   (FAT32 strategy)
    └── device_detector
```

---

## Getting Started

### Prerequisites

A stable Rust toolchain and the following system libraries (required to compile the GUI):

```bash
sudo apt-get install -y \
  libxkbcommon-dev \
  libwayland-dev \
  libx11-dev \
  libgtk-3-dev \
  pkg-config
```

### Build

```bash
git clone https://github.com/IgorSAssis/rust-recovery.git
cd rust-recovery

# Build everything
cargo build --release

# Run the desktop UI
./target/release/rust-recovery-ui

# Run the CLI
./target/release/cli --help
```

---

## Usage

### Desktop UI

Launch the application and use the sidebar to navigate between screens:

**Devices** — click *Detect Devices* to list available storage devices. Select one to automatically populate the source path on the Scan screen.

**Scan** — enter the path to a device (e.g. `/dev/sdb1`) or a disk image (e.g. `/path/to/image.img`), choose a recovery strategy, and click *Scan*. Results appear automatically when the scan completes.

**Recover** — browse the list of found files, click any entry to preview it (images are rendered inline), check the files you want to keep, and click *Export Selected* to save them to a folder of your choice.

---

### CLI

```bash
# List available storage devices
cli devices

# Scan a source and list found files (does not extract)
cli scan --source /dev/sdb1
cli scan --source /path/to/image.img

# Recover files to an output directory
cli recover --source /dev/sdb1 --output ./recovered

# Recover only specific file types
cli recover --source image.img --output ./recovered --types jpg,png,pdf

# Choose the FAT32 strategy
cli recover --source fat32.img --output ./recovered --strategy fat32

# Inspect raw bytes at a given offset
cli hexdump --source image.img --offset 4096 --length 256
```

---

## Project Structure

```
.
├── apps/
│   ├── cli/
│   │   └── src/
│   │       ├── main.rs
│   │       ├── app.rs             # CLI entry point and subcommand routing
│   │       └── commands/          # scan, recover, hexdump, devices
│   └── ui/
│       └── src/
│           ├── main.rs
│           ├── app.rs             # Application state and update logic (TEA model)
│           ├── message.rs         # All user-facing events
│           ├── worker.rs          # Async bridge to recovery_engine
│           ├── locale.rs          # i18n strings (en, pt-BR)
│           ├── notification.rs    # Toast notification model
│           └── views/             # Screen components (scan, results, devices, console…)
├── crates/
│   ├── device_detector/
│   │   └── src/linux.rs           # /sys/block parsing
│   ├── file_carver/
│   │   └── src/
│   │       ├── signature.rs       # JPEG, PNG, PDF, ZIP byte signatures
│   │       ├── scanner.rs         # Streaming scan with chunk overlap
│   │       └── extractor.rs       # Byte extraction from source
│   └── recovery_engine/
│       └── src/
│           ├── engine.rs          # RecoveryEngine — public API
│           ├── strategies/        # RecoveryStrategy trait, Carver, FAT32
│           ├── filesystems/fat32/ # Boot sector, directory entries, cluster navigation
│           └── types.rs           # ExtractedFile, FileInfo, StrategyKind
├── .github/
│   └── workflows/
│       ├── ci.yml                 # Lint, test and build on every push
│       └── release.yml            # Build release binaries on version tags
└── docs/                          # Development notes and planning documents
```

---

## CI/CD

Every push to `main` and every pull request runs the full CI pipeline:

1. `cargo fmt --check` — formatting
2. `cargo clippy -- -D warnings` — linting
3. `cargo test --workspace` — tests
4. `cargo build --workspace` — compilation

Pushing a version tag (e.g. `git tag v1.0.0 && git push origin v1.0.0`) triggers the release workflow: CI runs again, release binaries are compiled, and a GitHub Release is created with a `.tar.gz` containing both binaries.

---

## Roadmap

- [ ] Windows support for device detection and GUI
- [ ] EXT4 filesystem strategy
- [ ] GIF and BMP signature support
- [ ] Parallel scan benchmark and tuning
- [ ] Deduplication when running multiple strategies on the same source
