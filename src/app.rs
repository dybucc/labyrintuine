//! Core application state and logic for the labyrinth game.

use color_eyre::eyre::Result;
use ratatui::DefaultTerminal;

use crate::{
    events,
    map::Map,
    pathfinding::AnimationManager,
    types::{MainMenuItem, Screen},
    ui,
};

/// Application state container for the labyrinth game.
///
/// This structure holds the state of the application, which is to say the structure from which
/// Ratatui will render the game and Crossterm events will help writing to.
pub struct App {
    /// Application exit flag.
    ///
    /// This field indicates whether the application should exit. It is set to `true` when the user
    /// wants to quit the game but it starts off `false`.
    pub(crate) exit: bool,
    /// Current screen being displayed to the user.
    ///
    /// This field holds the current screen of the game. It is used to determine which screen to
    /// render and what actions to take based on user input.
    pub(crate) screen: Screen,
    /// Currently active labyrinth map.
    ///
    /// This field holds the current map of the game. It is used to render the labyrinth and solve
    /// it. The custom type always holds a map, either the default one or one loaded and selected by
    /// the user.
    pub(crate) map: Map,
    /// Collection of all available labyrinth maps.
    ///
    /// This field holds information about all the labyrinth maps in the current working directory.
    /// It consists of a key extracted straight from the filesystem and a vector with the contents
    /// of the map as string-rows, stored as custom types within an ordered collection.
    pub(crate) maps: Vec<Map>,
    /// Map currently selected in the viewport.
    ///
    /// This field holds the map that is currently selected in the viewport by the user cursor. This
    /// means the currently selected model in the maps menu.
    pub(crate) viewport_map: Option<Map>,
    /// Scrolling offset for the map list viewport.
    ///
    /// This field holds the offset by which to scroll the sliding window into the
    /// [`maps`](App::maps) vector in the maps menu's viewport.
    pub(crate) viewport_offset: usize,
    /// Height of the map list rendering area.
    ///
    /// This field holds the height of the area in which the list of maps are being rendered as a
    /// measure of terminal cells during the last redraw of the on-screen frame.
    pub(crate) viewport_height: usize,
    /// Animation manager for pathfinding visualization.
    ///
    /// This field manages the animation state including timing, current step tracking, and the
    /// coordinate path being displayed during the animated maze solving.
    pub(crate) animation_manager: AnimationManager,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Creates a new instance of the App structure with safe defaults.
    ///
    /// A [`Default`] trait implementation is not used here because the struct may perform a
    /// fallible operation in the future. The [`Default`] trait implementation does use this
    /// function, though.
    pub fn new() -> Self {
        Self {
            exit: false,
            screen: Screen::MainMenu(MainMenuItem::StartGame),
            map: Map::default(),
            maps: Vec::new(),
            viewport_map: None,
            viewport_offset: 0,
            viewport_height: 0,
            animation_manager: AnimationManager::new(),
        }
    }

    /// Runs the main loop of the application.
    ///
    /// This function handles user input and updates the application state. The loop continues until
    /// the exit condition is `true`, after which the function returns to the call site.
    ///
    /// # Errors
    ///
    /// - [`std::io::Error`]
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            let _ = terminal.try_draw(|frame| {
                ui::draw(self, frame)
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
            })?;
            events::handle_events(self)?;
        }

        Ok(())
    }
}
