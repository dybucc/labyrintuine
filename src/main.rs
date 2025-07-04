//! This crate contains the source code for the binary for the game labyrintuine.

#![expect(
    unused_crate_dependencies,
    reason = "The dependencies are used in the library crate."
)]

use std::path::PathBuf;

use clap::Parser;
use color_eyre::{eyre::Result, install};
use labyrintuine::App;

/// A terminal-based maze generator and solver with a user interface.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to a specific maze file to load
    #[arg(short, long, value_name = "FILE")]
    map: Option<PathBuf>,
}

fn main() -> Result<()> {
    install()?;

    let args = Args::parse();

    let mut terminal = ratatui::init();
    App::new_with_map(args.map)?.run(&mut terminal)?;
    ratatui::restore();

    Ok(())
}
