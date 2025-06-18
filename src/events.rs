//! Event handling functions for user input and application state updates.

use std::time::Duration;

use color_eyre::eyre::{OptionExt as _, Result};
use ratatui::crossterm::event::{self, Event, KeyCode};

use crate::{
    file_loader,
    map::Map,
    types::{MainMenuItem, OptionsMenuItem, Screen},
    App,
};

/// Handles input events and updates the application state accordingly.
///
/// This function polls for keyboard events and dispatches them to the appropriate handler
/// functions based on the key pressed. It uses a timeout to avoid blocking the UI.
pub(crate) fn handle_events(app: &mut App) -> Result<()> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => app.exit = true,
                KeyCode::Char('j') => handle_j_events(app)?,
                KeyCode::Char('k') => handle_k_events(app)?,
                KeyCode::Char('l') => handle_l_events(app)?,
                KeyCode::Char('h') => handle_h_events(app),
                _ => {}
            }
        }
    }

    // Update animation if in-game
    if matches!(app.screen, Screen::InGame) {
        app.animation_manager.update();
    }

    Ok(())
}

/// Handles 'j' key press events for downward navigation.
///
/// This function processes the 'j' key press which is used for moving down in menus and lists.
/// The behavior varies depending on the current screen, handling menu navigation and viewport
/// scrolling appropriately.
pub(crate) fn handle_j_events(app: &mut App) -> Result<()> {
    match app.screen {
        Screen::MainMenu(MainMenuItem::StartGame) => {
            app.screen = Screen::MainMenu(MainMenuItem::Options);
        }
        Screen::MainMenu(MainMenuItem::Options) => {
            app.screen = Screen::MainMenu(MainMenuItem::Quit);
        }
        Screen::OptionsMenu(OptionsMenuItem::Map) => {
            app.screen = Screen::OptionsMenu(OptionsMenuItem::Back);
        }
        Screen::MapMenu => {
            let viewport_map = app
                .viewport_map
                .clone()
                .ok_or_eyre("failed to retrieve cursor-selected map")?;

            if viewport_map
                == app
                    .maps
                    .iter()
                    .skip(app.viewport_offset)
                    .take(app.viewport_height)
                    .next_back()
                    .ok_or_eyre("no last element in viewport maps")?
                    .clone()
                && viewport_map
                    != app
                        .maps
                        .last()
                        .ok_or_eyre("failed to retrieve last map")?
                        .clone()
            {
                app.viewport_offset += 1;
            }

            let mut index = 0;
            for (idx, map) in app.maps.iter().enumerate() {
                if viewport_map == *map {
                    index = idx;
                    break;
                }
            }
            match app.maps.get(index + 1) {
                None => {}
                Some(element) => {
                    app.viewport_map = Some(element.clone());
                }
            }
        }
        _ => {}
    }

    Ok(())
}

/// Handles 'k' key press events for upward navigation.
///
/// This function processes the 'k' key press which is used for moving up in menus and lists.
/// Like the 'j' handler, behavior varies by screen and includes proper viewport management for
/// scrollable content.
pub(crate) fn handle_k_events(app: &mut App) -> Result<()> {
    match app.screen {
        Screen::MainMenu(MainMenuItem::Quit) => {
            app.screen = Screen::MainMenu(MainMenuItem::Options);
        }
        Screen::MainMenu(MainMenuItem::Options) => {
            app.screen = Screen::MainMenu(MainMenuItem::StartGame);
        }
        Screen::OptionsMenu(OptionsMenuItem::Back) => {
            app.screen = Screen::OptionsMenu(OptionsMenuItem::Map);
        }
        Screen::MapMenu => {
            let viewport_map = app
                .viewport_map
                .clone()
                .ok_or_eyre("failed to retrieve cursor-selected map")?;

            if viewport_map
                == app
                    .maps
                    .iter()
                    .skip(app.viewport_offset)
                    .take(app.viewport_height)
                    .cloned()
                    .collect::<Vec<Map>>()
                    .first()
                    .ok_or_eyre("no first element in viewport maps")?
                    .clone()
                && viewport_map
                    != app
                        .maps
                        .first()
                        .ok_or_eyre("failed to retrieve first map")?
                        .clone()
            {
                app.viewport_offset -= 1;
            }

            let mut index = 0;
            for (idx, map) in app.maps.iter().enumerate() {
                if viewport_map == *map {
                    index = idx;
                    break;
                }
            }
            if let Some(element) = app.maps.get(index.saturating_sub(1)) {
                app.viewport_map = Some(element.clone());
            }
        }
        _ => {}
    }

    Ok(())
}

/// Handles 'l' key press events for selection and forward navigation.
///
/// This function processes the 'l' key press which is used for selecting menu items and moving
/// forward in the application flow. It handles screen transitions, map loading, and selection
/// confirmation across different contexts.
pub(crate) fn handle_l_events(app: &mut App) -> Result<()> {
    match app.screen {
        Screen::MainMenu(MainMenuItem::StartGame) => {
            app.screen = Screen::InGame;
        }
        Screen::MainMenu(MainMenuItem::Options) => {
            app.screen = Screen::OptionsMenu(OptionsMenuItem::Map);
        }
        Screen::MainMenu(MainMenuItem::Quit) => {
            app.exit = true;
        }
        Screen::OptionsMenu(OptionsMenuItem::Map) => {
            app.screen = Screen::MapMenu;

            let first = Map::default();
            app.maps.clear();
            app.maps.push(first.clone());
            file_loader::fetch_files(&mut app.maps)?;
            app.viewport_map = Some(first);
            app.viewport_offset = 0;
        }
        Screen::OptionsMenu(OptionsMenuItem::Back) => {
            app.screen = Screen::MainMenu(MainMenuItem::StartGame);
        }
        Screen::MapMenu => {
            app.map = app
                .viewport_map
                .clone()
                .ok_or_eyre("failed to retrieve cursor-selected map")?;
        }
        _ => {}
    }

    Ok(())
}

/// Handles 'h' key press events for backward navigation.
///
/// This function processes the 'h' key press which is used for moving back or returning to
/// previous screens. It handles returning from the in-game screen to the main menu and from the
/// map menu to the options menu.
pub(crate) fn handle_h_events(app: &mut App) {
    match app.screen {
        Screen::InGame => {
            // Reset animation state and return to main menu
            app.animation_manager.clear();
            app.screen = Screen::MainMenu(MainMenuItem::StartGame);
        }
        Screen::MapMenu => {
            app.screen = Screen::OptionsMenu(OptionsMenuItem::Map);
        }
        _ => {}
    }
}
