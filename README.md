# guit

> Minimalist Git TUI (Terminal User Interface) client with AI support

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org)

*Leer en [Español](README.es.md)*

**guit** is an intuitive and lightweight terminal interface for Git that lets you manage your repositories without leaving the command line. Navigate changes, stage files, view diffs, manage branches, and create commits, all from an elegant text-based interface.

## Features

- **Intuitive navigation**: Use Vim-style keyboard shortcuts to move quickly
- **Real-time change view**: Visualize modified, staged, and unstaged files
- **Integrated diff**: Side panel with difference highlighting
- **Stage management**: Stage/unstage individual files or all at once
- **Interactive commits**: Write commit messages directly in the interface
- **AI support**: Automatically generate commit messages (Tab in commit mode)
- **Commit history**: Explore Git log without leaving the application
- **Branch management**: Switch between branches or create new ones from the interface
- **Lightweight and fast**: Built in Rust with minimal dependencies

## Installation

### Download precompiled binaries

Download the latest version from [Releases](https://github.com/yisas97/guit/releases):

| Platform | File |
|----------|------|
| Windows x64 | `guit-windows-x64.exe` |
| Linux x64 | `guit-linux-x64` |
| macOS Intel | `guit-macos-x64` |
| macOS Apple Silicon | `guit-macos-arm64` |

**Linux/macOS:** After downloading, give it execution permissions:
```bash
chmod +x guit-linux-x64
./guit-linux-x64
```

### From source

```bash
# Clone the repository
git clone https://github.com/yisas97/guit.git
cd guit

# Build and install
cargo install --path .
```

### Requirements

- Rust 1.70+ (2021 edition)
- Git installed on the system

## Usage

Run `guit` inside any Git repository:

```bash
cd your-git-project
guit
```

## Keyboard shortcuts

### Normal Mode

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down in file list |
| `k` / `↑` | Move up in file list |
| `h` / `l` / `Tab` | Toggle between panels (files/diff) |
| `Space` / `Enter` | Stage/unstage selected file |
| `a` | Stage all files |
| `u` | Unstage all files |
| `d` | Discard changes of selected file |
| `c` | Enter commit mode |
| `g` | View commit history (git log) |
| `b` | View and switch branches |
| `r` | Refresh status |
| `PgUp` / `PgDn` | Fast scroll in diff |
| `q` / `Esc` | Quit |

### Commit Mode

| Key | Action |
|-----|--------|
| `Type` | Enter commit message |
| `Tab` | Generate message with AI |
| `Enter` | Confirm commit |
| `Esc` | Cancel and return |
| `Backspace` | Delete character |
| `←` / `→` | Move cursor |

### Log Mode

| Key | Action |
|-----|--------|
| `j` / `↓` | Next commit |
| `k` / `↑` | Previous commit |
| `q` / `Esc` | Return to normal mode |

### Branches Mode

| Key | Action |
|-----|--------|
| `j` / `↓` | Next branch |
| `k` / `↑` | Previous branch |
| `Enter` / `Space` | Switch to selected branch |
| `n` | Create new branch |
| `q` / `Esc` | Return to normal mode |

### Create Branch Mode

| Key | Action |
|-----|--------|
| `Type` | New branch name |
| `Enter` | Create branch |
| `Esc` | Cancel |
| `Backspace` | Delete character |
| `←` / `→` | Move cursor |

## Project structure

```
guit/
├── src/
│   ├── main.rs      # Entry point and event loop
│   ├── app.rs       # Application state logic
│   ├── git.rs       # Git commands interface
│   └── ui.rs        # TUI interface rendering
├── Cargo.toml       # Project configuration
└── Cargo.lock       # Locked dependencies
```

## Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) - TUI framework for Rust
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation

## Development

```bash
# Run in development mode
cargo run

# Build optimized version
cargo build --release

# Run tests
cargo test
```

## Roadmap

- [ ] Full AI integration for commit message generation
- [ ] Support for merge conflict resolution
- [ ] Interactive stash
- [ ] File search and filtering
- [ ] Customizable themes
- [ ] Submodules support

## Contributing

Contributions are welcome. Please:

1. Fork the repository
2. Create a branch for your feature (`git checkout -b feature/new-feature`)
3. Commit your changes (`git commit -am 'Add new feature'`)
4. Push to the branch (`git push origin feature/new-feature`)
5. Open a Pull Request

## License

This project is under the MIT License. See the `LICENSE` file for more details.

## Author

**Jesus Campos**

- GitHub: [@yisas97](https://github.com/yisas97)

---

Built with Rust
