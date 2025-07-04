//! Core application state and logic for the labyrinth game.

use std::{fs, path::PathBuf};

use color_eyre::eyre::{OptionExt as _, Result};
use ratatui::DefaultTerminal;

use crate::{
    events, file_loader,
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

    /// Creates a new instance of the App with an optionally specified map file.
    ///
    /// If a map file is provided, it will be loaded and the game will start directly in the
    /// in-game screen. If no map file is provided or if loading fails, it falls back to the
    /// default behavior.
    ///
    /// # Errors
    ///
    /// This function may return errors from file operations or map validation.
    pub fn new_with_map(map_file: Option<PathBuf>) -> Result<Self> {
        let mut app = Self::new();

        if let Some(path) = map_file {
            // Load the specified map file
            let contents = fs::read_to_string(&path)?;
            
            // Validate the map format
            if file_loader::parse_file_contents(contents.trim()) {
                let filename = path
                    .file_name()
                    .ok_or_eyre("failed to extract filename from path")?
                    .to_owned();
                let map = Map::new(filename, &contents)?;
                
                // Set the loaded map as the current map and start the game
                app.map = map;
                app.screen = Screen::InGame;
            } else {
                return Err(color_eyre::eyre::eyre!("Invalid maze file format"));
            }
        }

        Ok(app)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_new_initialization() {
        let app = App::new();

        assert!(!app.exit);
        assert_eq!(app.screen, Screen::MainMenu(MainMenuItem::StartGame));
        assert_eq!(app.map, Map::default());
        assert!(app.maps.is_empty());
        assert!(app.viewport_map.is_none());
        assert_eq!(app.viewport_offset, 0);
        assert_eq!(app.viewport_height, 0);
    }

    #[test]
    fn test_app_default_implementation() {
        let app = App::default();

        assert!(!app.exit);
        assert_eq!(app.screen, Screen::MainMenu(MainMenuItem::StartGame));
        assert_eq!(app.map, Map::default());
        assert!(app.maps.is_empty());
        assert!(app.viewport_map.is_none());
        assert_eq!(app.viewport_offset, 0);
        assert_eq!(app.viewport_height, 0);
    }

    #[test]
    fn test_app_new_equals_default() {
        let app_new = App::new();
        let app_default = App::default();

        assert_eq!(app_new.exit, app_default.exit);
        assert_eq!(app_new.screen, app_default.screen);
        assert_eq!(app_new.map, app_default.map);
        assert_eq!(app_new.maps.len(), app_default.maps.len());
        assert_eq!(app_new.viewport_map, app_default.viewport_map);
        assert_eq!(app_new.viewport_offset, app_default.viewport_offset);
        assert_eq!(app_new.viewport_height, app_default.viewport_height);
    }

    #[test]
    fn test_app_exit_flag_modification() {
        let mut app = App::new();
        assert!(!app.exit);

        app.exit = true;
        assert!(app.exit);
    }

    #[test]
    fn test_app_screen_modification() {
        let mut app = App::new();
        assert_eq!(app.screen, Screen::MainMenu(MainMenuItem::StartGame));

        app.screen = Screen::InGame;
        assert_eq!(app.screen, Screen::InGame);
    }

    #[test]
    fn test_app_viewport_offset_modification() {
        let mut app = App::new();
        assert_eq!(app.viewport_offset, 0);

        app.viewport_offset = 5;
        assert_eq!(app.viewport_offset, 5);
    }

    #[test]
    fn test_app_viewport_height_modification() {
        let mut app = App::new();
        assert_eq!(app.viewport_height, 0);

        app.viewport_height = 10;
        assert_eq!(app.viewport_height, 10);
    }

    #[test]
    fn test_app_new_with_map_none() {
        let app = App::new_with_map(None).expect("Should create app with no map");
        
        assert!(!app.exit);
        assert_eq!(app.screen, Screen::MainMenu(MainMenuItem::StartGame));
        assert_eq!(app.map, Map::default());
    }

    #[test]
    fn test_app_new_with_map_nonexistent_file() {
        let result = App::new_with_map(Some(std::path::PathBuf::from("nonexistent.labmap")));
        
        assert!(result.is_err());
    }

    #[test]
    fn test_app_new_with_map_valid_file() {
        // Create a temporary valid maze file
        let temp_file = "/tmp/test_maze_valid.labmap";
        std::fs::write(temp_file, "2222\n2134\n2222").expect("Failed to write test file");
        
        let app = App::new_with_map(Some(std::path::PathBuf::from(temp_file)))
            .expect("Should create app with valid map");
        
        assert!(!app.exit);
        assert_eq!(app.screen, Screen::InGame);
        assert_eq!(app.map.key, "test_maze_valid");
        assert_eq!(app.map.data.len(), 3);
        
        // Clean up
        let _ = std::fs::remove_file(temp_file);
    }

    #[test]
    fn test_app_new_with_map_invalid_format() {
        // Create a temporary invalid maze file
        let temp_file = "/tmp/test_maze_invalid.labmap";
        std::fs::write(temp_file, "invalid\nmaze\nformat").expect("Failed to write test file");
        
        let result = App::new_with_map(Some(std::path::PathBuf::from(temp_file)));
        
        assert!(result.is_err());
        
        // Clean up
        let _ = std::fs::remove_file(temp_file);
    }
}
