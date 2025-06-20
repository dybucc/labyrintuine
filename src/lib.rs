//! Labyrintuine - A TUI maze game with pathfinding visualization.
//!
//! This library provides a terminal-based labyrinth game with animated pathfinding solutions. The
//! game features a modular architecture with separate concerns for UI rendering, event handling,
//! file operations, and core application logic.

mod app;
mod events;
mod file_loader;
mod map;
mod pathfinding;
mod types;
mod ui;

pub use app::App;
