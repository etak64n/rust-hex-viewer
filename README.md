# Rust Hex Viewer

A lightweight Rust-based hex viewer prototype that can be run either in TUI mode (ratatui + crossterm) or GUI mode (egui/eframe). The project currently focuses on fast read-only browsing of binary files while laying the groundwork for future editing features.

## Getting Started

```bash
git clone <repo>
cd rust-hex-viewer
cargo run -- <FILE>
```

### TUI Mode (default)

```
cargo run -- <FILE>
```

Key bindings:

- `j` / `k`: scroll one row down/up
- `Space` or `PageDown`: page down
- `PageUp`: page up
- `g` / `G`: jump to start/end
- `q` or `Esc`: quit

### GUI Mode (egui/eframe)

```
cargo run -- <FILE> --gui
```

Features:

- Scrollable hex/ASCII view rendered with egui.
- Click any byte (hex or ASCII cell) or move with arrow keys to highlight it; the status bar shows the selected offset.
- Pass `--debug` to print scroll/selection debug logs to stderr.

## Status

This is an early prototype. The next milestones are:

1. Add search and jump-to-offset capabilities.
2. Introduce basic editing (insert/overwrite) with undo/redo.
3. Reuse the shared core (`App`) for both the TUI and GUI as features expand.
