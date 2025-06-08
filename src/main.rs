//! This crate contains the source code for the binary for the game labyrintuine.

#![expect(
    clippy::cargo_common_metadata,
    reason = "Temporary allow during development."
)]
#![expect(
    unused_crate_dependencies,
    reason = "The dependencies are used in the library crate."
)]

use color_eyre::{eyre::Result, install};
use labyrintuine::App;

fn main() -> Result<()> {
    install()?;

    let mut terminal = ratatui::init();
    App::default().run(&mut terminal)?;
    ratatui::restore();

    Ok(())
}
