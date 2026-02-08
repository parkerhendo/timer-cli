# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Troubleshooting

Always check `docs/` first when encountering errors or issues. Solutions are organized by category (runtime-errors, logic-errors, etc.) with YAML frontmatter for searchability.

## Build Commands

**Rust CLI:**

```sh
cargo build              # debug build
cargo build --release    # release build
cargo install --path .   # install both `timer` and `timer-cli` binaries
```

**Swift menu bar app (macOS 13+):**

```sh
cd TimerBar
swift build              # debug build
swift build -c release   # release build
```

## Architecture

This is a time tracking tool with two interfaces sharing a single SQLite database:

1. **Rust CLI** (`src/`) - Command-line interface using clap for argument parsing
2. **Swift TimerBar** (`TimerBar/`) - macOS menu bar companion app using SwiftUI

### Shared Database

Both interfaces read/write to `~/Library/Application Support/timer-cli/frames.db` (configurable via `TIMER_CLI_DB` env var). WAL mode is enabled for concurrent access.

Schema:

```sql
frames(id INTEGER PRIMARY KEY, project TEXT, start_time INTEGER, end_time INTEGER, tags TEXT)
```

- Timestamps are Unix epoch seconds
- Tags are comma-separated strings
- A frame with `end_time IS NULL` is the active timer

### Rust Structure

- `main.rs` - CLI argument parsing, routes to command handlers
- `db.rs` - Database connection, schema initialization, WAL mode config
- `frame.rs` - Frame data model, duration formatting, timestamp conversion helpers
- `commands/*.rs` - Individual subcommand implementations (start, stop, log, report, etc.)

### Swift Structure

- `TimerBarApp.swift` - App entry point with MenuBarExtra
- `TimerState.swift` - Observable state manager, polls DB every second + watches file changes
- `TimerDatabase.swift` - SQLite wrapper using SQLite.swift library
- `MenuView.swift` - Menu UI for tracking/idle states
- `Frame.swift` - Data model mirroring Rust version

### Key Implementation Details

- DST handling: Use `timestamp_to_local()` in Rust which handles `LocalResult::Ambiguous` cases
- Tags: Parsed with `+` prefix in CLI (`+coding`), stored without prefix
- Both binaries (`timer` and `timer-cli`) point to same `src/main.rs`
