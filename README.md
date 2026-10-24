# `ftop` 📂⚡

A high-performance, real-time terminal directory activity monitor (like `top` or `htop` for processes) that tracks active filesystem changes across directories.

Built with **Rust**, [`ratatui`](https://ratatui.rs/), [`crossterm`](https://crates.io/crates/crossterm), and [`notify`](https://crates.io/crates/notify).

---

## Features

- **Real-Time Recursive Monitoring**: Uses native OS kernel APIs (`ReadDirectoryChangesW` on Windows, `inotify` on Linux, `FSEvents` on macOS).
- **Zero-Lag Event Ingestion**: Drains filesystem change events directly into an in-memory registry, debounced naturally against the 30 FPS terminal frame rate.
- **Top-like Sorting**:
  - `Most Recent`: Shows files in order of latest modification/creation/deletion.
  - `Highest Changes`: Highlights "hot" files with high mutation rates.
  - `File Size`: Shows largest changing files.
  - `Path Alphabetical`: Alphabetical ordering.
- **Smart Filtering**: Automatically ignores noisy directories (`.git`, `node_modules`, `target`, `.cache`) and temporary swap files (can be inspected with `--all`).
- **Safe Terminal Recovery**: Automatically restores your terminal screen even if interrupted or upon panic.

---

## Installation

### From crates.io (Once published)
```bash
cargo install ftop
```

### From GitHub Repository
```bash
cargo install --git https://github.com/<your-username>/ftop
```

### Local Build
```bash
git clone https://github.com/<your-username>/ftop.git
cd ftop
cargo build --release
```

---

## Usage

```bash
# Watch current directory
ftop

# Watch a specific directory
ftop path/to/project

# Include hidden/ignored directories (.git, node_modules)
ftop . --all
```

*(Note: Both `ftop` and `filetop` command aliases are supported!)*

---

## Keybindings

| Key | Action |
| :--- | :--- |
| `q` or `Esc` | Exit `ftop` |
| `Space` | **Pause / Resume** real-time screen updates |
| `/` | **Interactive search/filter** by path or file extension |
| `Enter` | **Open selected file** in default system editor or application |
| `s` | Cycle sort column (`Recent` → `Changes` → `Size` → `Path`) |
| `c` | Clear current event history |
| `↑` / `k` | Navigate selection up |
| `↓` / `j` | Navigate selection down |

---

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
