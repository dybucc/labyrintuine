//! Type definitions and enums for the application state and navigation.

/// Enumeration of available application screens.
///
/// This enumeration holds information about the current screen of the game. This is used to
/// determine which screen to render and what actions to take based on user input.
pub(crate) enum Screen {
    /// Main menu screen of the game.
    ///
    /// This variant represents the main menu screen of the game.
    MainMenu(MainMenuItem),
    /// Options configuration screen.
    ///
    /// This variant represents the options menu screen of the game.
    OptionsMenu(OptionsMenuItem),
    /// In-game maze visualization screen.
    ///
    /// This variant represents the ingame screen where the labyrinth is displayed and solved.
    InGame,
    /// Map selection screen.
    ///
    /// This variant represents the map menu screen of the game. It contains a list of the maps
    /// available to the user.
    MapMenu,
}

/// Main menu navigation options.
///
/// This enumeration holds the different items in the main menu. It is used to determine which items
/// can the user select in the main menu.
#[derive(Clone, Copy)]
pub(crate) enum MainMenuItem {
    /// "Start Game" menu option.
    ///
    /// This variant represents the "Start Game" option in the main menu.
    StartGame,
    /// "Options" menu option.
    ///
    /// This variant represents the "Options" option in the main menu.
    Options,
    /// "Quit" menu option.
    ///
    /// This variant represents the "Quit" option in the main menu.
    Quit,
}

/// Options menu navigation choices.
///
/// This enumeration holds the different items in the options menu. It is used to determine which
/// items can the user select in the options menu.
#[derive(Clone, Copy)]
pub(crate) enum OptionsMenuItem {
    /// "Back" navigation option.
    ///
    /// This variant represents the "Back" option in the options menu.
    Back,
    /// "Map" selection option.
    ///
    /// This variant represents the "Map" option in the options menu.
    Map,
}

/// Generic menu type configuration.
///
/// This enumeration holds the different specifics particular to each generic menu type in the
/// application's interface. Generic here means they share enough features to be considered worth
/// joining together part of their functionality.
pub(crate) enum MenuType {
    /// Main menu configuration.
    ///
    /// This variant represents the main menu in the game.
    MainMenu(u8),
    /// Options menu configuration.
    ///
    /// This variant represents the options menu in the game.
    OptionsMenu(u8),
}

impl MenuType {
    /// Returns the string representation of the menu type.
    ///
    /// This function provides the display name for each menu variant, used as the title in the
    /// menu's border when rendering the interface.
    pub(crate) const fn repr(&self) -> &str {
        match self {
            Self::MainMenu(_) => "Main Menu",
            Self::OptionsMenu(_) => "Options Menu",
        }
    }

    /// Returns the numeric value stored by the menu type variant.
    ///
    /// This function provides access to the number of menu items for layout calculations, allowing
    /// the UI to properly size the menu containers.
    pub(crate) const fn value(&self) -> u8 {
        match self {
            Self::MainMenu(value) => *value,
            Self::OptionsMenu(value) => *value,
        }
    }
}
