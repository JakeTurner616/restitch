<a id="readme-top"></a>

<h1 align="center">🧵 RESTITCH</h1>
<p align="center"><i>Reliable config backup, restore, and revert — from the terminal, in Rust.</i></p>

<p align="center">
  <img src="https://img.shields.io/badge/built_with-rust-orange?style=flat-square&logo=rust" />
  <img src="https://img.shields.io/badge/platform-linux%20%26%20macOS-green?style=flat-square" />
  <img src="https://img.shields.io/badge/interface-tui-blue?style=flat-square" />
</p>

---

## Overview

**Restitch** is a simple command-line utility for organizing configs and dotfiles — backup, restore, and revert system configurations.

No more manually backing up and moving files between installs. Restitch helps you **track, package, and restore your setup across machines**.

---

## Usage

```bash
restitch                   # Launch the TUI to select configs and create a backup
restitch --restore --dry-run   # Preview the restore without applying changes
restitch --restore         # Restore from the most recent archive + manifest
restitch --revert          # Revert to the last backup (interactive prompt)
restitch --help            # CLI reference
```

### CLI Flags

| Flag                   | Description                             |
| ---------------------- | --------------------------------------- |
| `--archive <path>`     | Use a specific archive `.tar.gz` file   |
| `--manifest <path>`    | Use a specific manifest `.toml` file    |
| `--config-path <path>` | Load custom config targets `.toml` file |

---

## Configuration Format

Create a `config_targets.toml` in the working directory (or specify one via `--config-path`):

```toml
[[config]]
name = "Zsh Config"
path = "~/.zshrc"

[[config]]
name = "Kitty Terminal"
path = "~/.config/kitty"
```

Tilde (`~`) is supported. Only existing files or directories will be included.

---

## Dry Run Preview

```bash
restitch --restore --dry-run
```

Outputs a detailed restore plan **without modifying** your system:

```text
🧭 Restore Plan (dry-run):
───────────────────────────────────────────────
🔁 REPLACE: Zsh Config → ~/.zshrc
🔁 REPLACE: Kitty Terminal → ~/.config/kitty

🔎 Restore dry-run complete.
👉 To apply these changes, run with `--restore`
```

---

## 🔁 Revert System Configs

To roll back to the most recent state before a restore:

```bash
restitch --revert
```

Prompts you before overwriting current files with a backup from `./backups/`.

---

## Output Structure

| Path                                     | Description                     |
| ---------------------------------------- | ------------------------------- |
| `outputs/restitch-archive.tar.gz`        | Generated config archive        |
| `outputs/restitch-archive.manifest.toml` | Manifest listing included files |
| `backups/YYYY-MM-DD_HH-MM-SS/`           | Auto-backups before restore     |


## Install

### Option 1: Use Prebuilt Linux Binaries

Visit the [Releases Page](https://github.com/JakeTurner616/restitch/releases) and download the archive for your platform:

```bash
tar -xzf restitch-v0.1.1-x86_64-unknown-linux-gnu.tar.gz
sudo cp restitch-v0.1.1-x86_64-unknown-linux-gnu/restitch /usr/local/bin/
```

> Also available for: `aarch64-unknown-linux-gnu`.

### Option 2: Build from Source

```bash
git clone https://github.com/JakeTurner616/restitch.git
cd restitch
cargo build --release
sudo cp target/release/restitch /usr/local/bin/
```

> Requires [Rust](https://rustup.rs) to build from source.

---


## License

MIT License — free to use, modify, and distribute.

> Built for terminal users who actually care about their configs.