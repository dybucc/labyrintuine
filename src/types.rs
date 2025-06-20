//! Type definitions and enums for the application state and navigation.

/// Enumeration of available application screens.
///
/// This enumeration holds information about the current screen of the game. This is used to
/// determine which screen to render and what actions to take based on user input.
#[derive(Debug, PartialEq)]
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
#[derive(Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Copy, Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screen_variants() {
        let main_menu = Screen::MainMenu(MainMenuItem::StartGame);
        let options_menu = Screen::OptionsMenu(OptionsMenuItem::Back);
        let in_game = Screen::InGame;
        let map_menu = Screen::MapMenu;

        assert_eq!(main_menu, Screen::MainMenu(MainMenuItem::StartGame));
        assert_eq!(options_menu, Screen::OptionsMenu(OptionsMenuItem::Back));
        assert_eq!(in_game, Screen::InGame);
        assert_eq!(map_menu, Screen::MapMenu);

        assert_ne!(main_menu, in_game);
        assert_ne!(options_menu, map_menu);
    }

    #[test]
    fn test_main_menu_item_variants() {
        let start_game = MainMenuItem::StartGame;
        let options = MainMenuItem::Options;
        let quit = MainMenuItem::Quit;

        assert_eq!(start_game, MainMenuItem::StartGame);
        assert_eq!(options, MainMenuItem::Options);
        assert_eq!(quit, MainMenuItem::Quit);

        assert_ne!(start_game, options);
        assert_ne!(options, quit);
        assert_ne!(start_game, quit);
    }

    #[test]
    fn test_options_menu_item_variants() {
        let back = OptionsMenuItem::Back;
        let map = OptionsMenuItem::Map;

        assert_eq!(back, OptionsMenuItem::Back);
        assert_eq!(map, OptionsMenuItem::Map);
        assert_ne!(back, map);
    }

    #[test]
    fn test_menu_type_repr() {
        let main_menu = MenuType::MainMenu(3);
        let options_menu = MenuType::OptionsMenu(2);

        assert_eq!(main_menu.repr(), "Main Menu");
        assert_eq!(options_menu.repr(), "Options Menu");
    }

    #[test]
    fn test_menu_type_value() {
        let main_menu = MenuType::MainMenu(3);
        let options_menu = MenuType::OptionsMenu(2);

        assert_eq!(main_menu.value(), 3);
        assert_eq!(options_menu.value(), 2);
    }

    #[test]
    fn test_menu_type_with_different_values() {
        let main_menu_5 = MenuType::MainMenu(5);
        let main_menu_10 = MenuType::MainMenu(10);
        let options_menu_0 = MenuType::OptionsMenu(0);

        assert_eq!(main_menu_5.value(), 5);
        assert_eq!(main_menu_10.value(), 10);
        assert_eq!(options_menu_0.value(), 0);

        assert_eq!(main_menu_5.repr(), "Main Menu");
        assert_eq!(main_menu_10.repr(), "Main Menu");
        assert_eq!(options_menu_0.repr(), "Options Menu");
    }

    #[test]
    fn test_debug_implementations() {
        let screen = Screen::InGame;
        let main_item = MainMenuItem::StartGame;
        let options_item = OptionsMenuItem::Back;

        assert_eq!(format!("{screen:?}"), "InGame");
        assert_eq!(format!("{main_item:?}"), "StartGame");
        assert_eq!(format!("{options_item:?}"), "Back");
    }

    #[test]
    fn test_clone_copy_traits() {
        let main_item = MainMenuItem::Options;
        let copied_main = main_item;
        let cloned_main = main_item;

        assert_eq!(main_item, copied_main);
        assert_eq!(main_item, cloned_main);

        let options_item = OptionsMenuItem::Map;
        let copied_options = options_item;
        let cloned_options = options_item;

        assert_eq!(options_item, copied_options);
        assert_eq!(options_item, cloned_options);
    }
}
