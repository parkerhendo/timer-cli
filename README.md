# timer-cli

CLI time tracker with SQLite backend.

## Installation

```sh
cargo install --path .
```

This installs both `timer-cli` and `timer` (shorter alias).

## Usage

```sh
# Start tracking
timer start myproject +coding +feature

# Check status
timer status

# Stop tracking
timer stop

# View log
timer log
timer log --from 2024-01-01 --to 2024-01-31

# Reports
timer report
timer report --by-tag
timer report --from 2024-01-01

# List projects and tags
timer projects
timer tags

# Edit a frame
timer edit 42 --project newname --tags +newtag

# Delete/cancel
timer cancel          # delete current frame
timer delete 42       # delete by ID

# Restart last stopped frame
timer restart

# Export data
timer export --format json
timer export --format csv

# Shell completions
timer completions bash >> ~/.bashrc
timer completions zsh >> ~/.zshrc
timer completions fish > ~/.config/fish/completions/timer.fish
```

## Data

Frames are stored in SQLite at `~/.local/share/timer-cli/timer.db` (or platform equivalent).
