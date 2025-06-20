<div align="center">

# 🌀 Labyrintuine

</div>

A terminal-based labyrinth game with animated pathfinding visualization built in Rust. Navigate
through maze structures and watch as the pathfinding algorithm explores and solves the labyrinth in
real-time.

## ✨ Features

- **Interactive TUI Interface**: Clean terminal-based user interface built with
  [Ratatui](https://ratatui.rs/)
- **Animated Pathfinding**: Watch depth-first search algorithm explore mazes with animated
  visualization
- **Custom Map Support**: Load your own `.labmap` files or use the built-in default maze
- **Cross-platform**: Runs on Linux, macOS, and Windows

## 🚀 Quick Start

### Prerequisites

- Rust toolchain (1.84+ recommended)
- Terminal with Unicode support

### Installation

```bash
# Clone the repository
git clone https://github.com/dybucc/labyrintuine.git
cd labyrintuine

# Build and run
cargo run --release
```

## 🎮 How to Play

1. **Main Menu**: Use arrow keys to navigate between options
2. **Start Game**: Launch the maze with pathfinding animation
3. **Load Maps**: Browse and select custom `.labmap` files from the directory where you launched the
   binary
4. **Watch the Magic**: Observe the depth-first search algorithm solve the maze

### Map Format

Maps use a simple text format (`.labmap` files):
- `1` - Entry point
- `2` - Walls
- `3` - Open paths
- `4` - Exit point

## 🔧 Development

### Building

```bash
# Debug build
cargo build

# Release build  
cargo build --release
```

## 📦 Dependencies

- **[ratatui](https://crates.io/crates/ratatui)** `0.29.0` - Terminal user interface library
- **[color-eyre](https://crates.io/crates/color-eyre)** `0.6.5` - Enhanced error reporting
- **[rounded-div](https://crates.io/crates/rounded-div)** `0.1.3` - Mathematical utilities

## 🤝 Contributing

Contributions are welcome! Please ensure your code:

1. Passes all existing tests: `cargo test`
2. Follows the linting rules: `cargo clippy`
3. Is properly formatted: `cargo fmt`
4. Includes comprehensive documentation

The project uses extensive linting rules to maintain high code quality. Check the `Cargo.toml` for
the complete list of enabled lints.

## 📄 License

This project is licensed under the Unlicense. See the LICENSE file for details.