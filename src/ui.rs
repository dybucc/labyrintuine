//! User interface rendering functions for all application screens.

use std::rc::Rc;

use color_eyre::eyre::{OptionExt as _, Result};
use ratatui::{
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Style},
    symbols::{Marker, DOT},
    text::Line,
    widgets::{
        canvas::{Canvas, Points},
        Block, BorderType, Borders, Clear,
    },
    Frame,
};

use crate::{
    map::Map,
    pathfinding,
    types::{MainMenuItem, MenuType, OptionsMenuItem, Screen},
    App,
};

/// Updates the application UI based on the persistent state.
///
/// This function renders different screens based on the current state stored in the [`App`]
/// structure, dispatching to the appropriate rendering function for each screen type.
///
/// # Errors
///
/// This function may return errors from drawing operations or data conversion failures.
pub(crate) fn draw(app: &mut App, frame: &mut Frame) -> Result<()> {
    match &app.screen {
        Screen::MainMenu(item) => main_menu(frame, *item),
        Screen::OptionsMenu(item) => options_menu(frame, *item),
        Screen::InGame => in_game(app, frame)?,
        Screen::MapMenu => map_menu(app, frame)?,
    }

    Ok(())
}

/// Clears the terminal screen by rendering a [`Clear`] widget.
///
/// This function renders a clear widget over the entire area of the frame to prepare for
/// rendering new content without artifacts from previous buffers rendered on the same frame.
pub(crate) fn clear(frame: &mut Frame) {
    let clear = Clear;
    frame.render_widget(clear, frame.area());
}

/// Renders the generic layout structure for the main and options menus.
///
/// This function creates the common layout and block structure used by both main and options menus.
/// The generic part includes the centered positioning and border styling, while the specific menu
/// content is handled by the caller using the [`MenuType`] parameter.
#[expect(
    clippy::indexing_slicing,
    reason = "The collection is created in-place with few, known elements; there is no risk of bad indexing."
)]
pub(crate) fn init_menu(frame: &mut Frame, menu: MenuType) -> Rc<[Rect]> {
    let space = Layout::vertical([
        Constraint::Percentage(40),
        Constraint::Percentage(20),
        Constraint::Percentage(40),
    ])
    .split(frame.area())[1];
    let space = Layout::horizontal([
        Constraint::Percentage(40),
        Constraint::Percentage(20),
        Constraint::Percentage(40),
    ])
    .split(space)[1];

    let layout = Layout::vertical([Constraint::Max(u16::from(menu.value() + 2))])
        .flex(Flex::Center)
        .split(space)[0];

    let block = Block::bordered()
        .title(menu.repr())
        .title_bottom("(j) down / (k) up / (l) select")
        .title_alignment(Alignment::Center)
        .style(Color::Green)
        .border_type(BorderType::Rounded);

    let inner_space = block.inner(layout);

    frame.render_widget(block, layout);

    Layout::vertical(vec![Constraint::Max(1); menu.value() as usize]).split(inner_space)
}

/// Renders the main menu screen with navigation options.
///
/// This function displays the main menu with options for "Start Game", "Options", and "Quit". It
/// highlights the currently selected option and provides visual feedback for user navigation.
#[expect(
    clippy::indexing_slicing,
    reason = "The collection is created in-place with few, known elements; there is no risk of bad indexing."
)]
#[expect(
    clippy::missing_asserts_for_indexing,
    reason = "The collection is created in-place with few, known elements; there is no risk of bad indexing."
)]
pub(crate) fn main_menu(frame: &mut Frame, item: MainMenuItem) {
    clear(frame);

    let inner_layout = init_menu(frame, MenuType::MainMenu(3));

    let content_style = Style::default().fg(Color::Green);
    let active_content_style = Style::default().fg(Color::White).bg(Color::Green);

    let mut opt1 = Line::raw("Start Game").centered();
    let mut opt2 = Line::raw("Options").centered();
    let mut opt3 = Line::raw("Quit").centered();
    match item {
        MainMenuItem::StartGame => {
            opt1 = opt1.style(active_content_style);
            opt2 = opt2.style(content_style);
            opt3 = opt3.style(content_style);
        }
        MainMenuItem::Options => {
            opt1 = opt1.style(content_style);
            opt2 = opt2.style(active_content_style);
            opt3 = opt3.style(content_style);
        }
        MainMenuItem::Quit => {
            opt1 = opt1.style(content_style);
            opt2 = opt2.style(content_style);
            opt3 = opt3.style(active_content_style);
        }
    }

    frame.render_widget(opt1, inner_layout[0]);
    frame.render_widget(opt2, inner_layout[1]);
    frame.render_widget(opt3, inner_layout[2]);
}

/// Renders the options menu screen with configuration choices.
///
/// This function displays the options menu with choices for "Map" selection and "Return" to the
/// main menu. It provides the same navigation highlighting as the main menu.
#[expect(
    clippy::indexing_slicing,
    reason = "The collection is created in-place with few, known elements; there is no risk of bad indexing."
)]
#[expect(
    clippy::missing_asserts_for_indexing,
    reason = "The collection is created in-place with few, known elements; there is no risk of bad indexing."
)]
pub(crate) fn options_menu(frame: &mut Frame, item: OptionsMenuItem) {
    clear(frame);

    let inner_layout = init_menu(frame, MenuType::OptionsMenu(2));

    let content_style = Style::default().fg(Color::Green);
    let active_content_style = Style::default().fg(Color::White).bg(Color::Green);

    let mut opt1 = Line::raw("Map").centered();
    let mut opt2 = Line::raw("Return").centered();
    match item {
        OptionsMenuItem::Map => {
            opt1 = opt1.style(active_content_style);
            opt2 = opt2.style(content_style);
        }
        OptionsMenuItem::Back => {
            opt1 = opt1.style(content_style);
            opt2 = opt2.style(active_content_style);
        }
    }

    frame.render_widget(opt1, inner_layout[0]);
    frame.render_widget(opt2, inner_layout[1]);
}

/// Renders the map selection menu with scrollable list of available maps.
///
/// This function displays a viewport containing all loadable maze maps from the current directory.
/// It provides scrolling functionality and visual indicators for the currently selected map and the
/// map that's actively being used.
///
/// # Errors
///
/// This function may return errors if the viewport map cannot be retrieved.
#[expect(
    clippy::indexing_slicing,
    reason = "The collection is created in-place with few, known elements; there is no risk of bad indexing."
)]
#[expect(
    clippy::missing_asserts_for_indexing,
    reason = "The collection is created in-place with few, known elements; there is no risk of bad indexing."
)]
pub(crate) fn map_menu(app: &mut App, frame: &mut Frame) -> Result<()> {
    clear(frame);

    let space = Layout::horizontal([
        Constraint::Percentage(30),
        Constraint::Fill(1),
        Constraint::Percentage(30),
    ])
    .split(frame.area())[1];
    let space = Layout::vertical([
        Constraint::Percentage(40),
        Constraint::Fill(1),
        Constraint::Percentage(40),
    ])
    .split(space)[1];

    let layout = Layout::vertical([Constraint::Min(1)]).split(space)[0];
    let block = Block::bordered()
        .title_top("Map list")
        .title_bottom("(j) down / (k) up / (l) select / (h) return")
        .title_alignment(Alignment::Center)
        .style(Color::Green)
        .border_type(BorderType::Rounded);

    let inner_space = block.inner(layout);

    frame.render_widget(block, layout);

    app.viewport_height = inner_space.height.into();

    let inner_layout = Layout::horizontal([Constraint::Percentage(5), Constraint::Percentage(100)])
        .split(inner_space);
    let inner_selector = Layout::vertical(vec![Constraint::Max(1); inner_space.height.into()])
        .split(inner_layout[0]);
    let inner_list = Layout::vertical(vec![Constraint::Max(1); inner_space.height.into()])
        .split(inner_layout[1]);

    let mut viewport_maps: Vec<&Map> = app.maps.iter().skip(app.viewport_offset).collect();
    viewport_maps.truncate(inner_space.height.into());

    let content_style = Style::default().fg(Color::Green);
    let active_content_style = Style::default().fg(Color::White).bg(Color::Green);

    for (idx, map) in viewport_maps.into_iter().enumerate() {
        let viewport_map = app
            .viewport_map
            .clone()
            .ok_or_eyre("failed to retrieve cursor-selected map")?;

        let (selector, entry) = if *map == viewport_map {
            (
                {
                    if *map == app.map {
                        Line::styled(DOT, active_content_style).centered()
                    } else {
                        Line::styled(" ", active_content_style).centered()
                    }
                },
                Line::styled(map.key.clone(), active_content_style),
            )
        } else {
            (
                {
                    if *map == app.map {
                        Line::styled(DOT, content_style).centered()
                    } else {
                        Line::styled(" ", content_style).centered()
                    }
                },
                Line::styled(map.key.clone(), content_style),
            )
        };

        frame.render_widget(selector, inner_selector[idx]);
        frame.render_widget(entry, inner_list[idx]);
    }

    Ok(())
}

/// Renders the in-game screen with maze visualization and pathfinding solution.
///
/// This function displays the currently selected labyrinth and runs the pathfinding algorithm to
/// show the solution. It renders both the maze walls and the computed paths using [`Canvas`]
/// widgets for precise coordinate-based drawing.
///
/// # Errors
///
/// This function may return errors from coordinate conversion operations or entry point
/// detection.
#[expect(
    clippy::too_many_lines,
    reason = "UI rendering function requires many lines for layout and drawing operations."
)]
pub(crate) fn in_game(app: &mut App, frame: &mut Frame) -> Result<()> {
    clear(frame);

    // Initialize animation steps if not already done
    if app.animation_manager.steps.is_empty() {
        // Find the maze entry point (marked with '1')
        let entry_point = app
            .map
            .data
            .iter()
            .enumerate()
            .find_map(|(row, line)| {
                line.bytes()
                    .enumerate()
                    .find_map(|(col, char)| (char == b'1').then_some((col, row)))
            })
            .ok_or_eyre("failed to retrieve entry point in map")?;

        // Record animation steps
        let mut initial_path = Vec::new();
        pathfinding::record_animation_steps(
            &app.map.data,
            entry_point,
            &mut initial_path,
            &mut app.animation_manager.steps,
        );

        app.animation_manager.reset();
    }

    let maze_rows = app.map.data.len();
    let maze_columns = app
        .map
        .data
        .first()
        .ok_or_eyre("failed to retrieve maze in selected map")?
        .len();

    // Create overall layout: maze area + tooltip at bottom
    let overall_layout = Layout::vertical([
        Constraint::Min(1),    // Maze and padding area
        Constraint::Length(3), // Tooltip block
    ])
    .split(frame.area());

    let maze_content_area = *overall_layout
        .first()
        .ok_or_eyre("failed to get maze content area from layout")?;
    let tooltip_full_area = *overall_layout
        .last()
        .ok_or_eyre("failed to get tooltip area from layout")?;

    // Center the tooltip horizontally like the maze
    let tooltip_area = Layout::horizontal([
        Constraint::Min(1),
        Constraint::Length(u16::try_from(maze_columns)?),
        Constraint::Min(1),
    ])
    .split(tooltip_full_area)
    .get(1)
    .copied()
    .ok_or_eyre("failed to get centered tooltip area from horizontal layout")?;

    // Create maze layout within the content area
    let main_layout = Layout::vertical([
        Constraint::Min(1),
        Constraint::Length(u16::try_from(maze_rows)?),
        Constraint::Min(1),
    ])
    .split(maze_content_area);

    let maze_area = main_layout
        .get(1)
        .ok_or_eyre("failed to get maze area from layout")?;

    let space = Layout::horizontal([
        Constraint::Min(1),
        Constraint::Length(u16::try_from(maze_columns)?),
        Constraint::Min(1),
    ])
    .split(*maze_area)
    .get(1)
    .copied()
    .ok_or_eyre("failed to get maze space from horizontal layout")?;

    // Pre-compute screen coordinates to handle errors before closures
    let mut wall_coords = Vec::new();
    for (row_idx, row) in app.map.data.iter().enumerate() {
        for (col_idx, cell) in row.bytes().enumerate() {
            if cell == b'2' {
                wall_coords.push((col_idx, row_idx));
            }
        }
    }
    let wall_screen_coords =
        pathfinding::transform_maze_to_screen_coords(&wall_coords, &app.map.data)?;
    let animation_screen_coords = pathfinding::transform_maze_to_screen_coords(
        &app.animation_manager.current_path,
        &app.map.data,
    )?;

    let maze = Canvas::default()
        .x_bounds([
            (-rounded_div::i32(space.width.into(), 2)).into(),
            (rounded_div::i32(space.width.into(), 2)).into(),
        ])
        .y_bounds([
            (-rounded_div::i32(space.height.into(), 2)).into(),
            (rounded_div::i32(space.height.into(), 2)).into(),
        ])
        .marker(Marker::Dot)
        .paint(|ctx| {
            // Render pre-computed wall coordinates
            ctx.draw(&Points {
                coords: &wall_screen_coords,
                color: Color::Green,
            });
        });
    let solution = Canvas::default()
        .x_bounds([
            (-rounded_div::i32(space.width.into(), 2)).into(),
            (rounded_div::i32(space.width.into(), 2)).into(),
        ])
        .y_bounds([
            (-rounded_div::i32(space.height.into(), 2)).into(),
            (rounded_div::i32(space.height.into(), 2)).into(),
        ])
        .marker(Marker::Dot)
        .paint(|ctx| {
            // Render pre-computed animation coordinates
            ctx.draw(&Points {
                coords: &animation_screen_coords,
                color: Color::Red,
            });
        });

    frame.render_widget(maze, space);
    frame.render_widget(solution, space);

    // Render tooltip as a block at the bottom center with top border
    let tooltip_block = Block::bordered()
        .title("(h) return to menu")
        .title_alignment(Alignment::Center)
        .style(Style::default().fg(Color::Green))
        .border_type(BorderType::Plain)
        .borders(Borders::TOP);

    frame.render_widget(tooltip_block, tooltip_area);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pathfinding::AnimationManager;
    use ratatui::{backend::TestBackend, Terminal};

    /// Creates a minimal test app for UI testing.
    fn create_test_app() -> App {
        App::new()
    }

    /// Creates a test terminal with known dimensions for UI testing.
    fn create_test_terminal() -> Terminal<TestBackend> {
        let backend = TestBackend::new(80, 24);
        Terminal::new(backend).expect("failed to create test terminal")
    }

    /// Creates a test map for UI testing.
    fn create_test_map() -> Map {
        Map {
            key: "test_map".to_owned(),
            data: vec![
                "222222222".to_owned(),
                "2100000002".to_owned(),
                "2020202002".to_owned(),
                "2000203002".to_owned(),
                "222222222".to_owned(),
            ],
        }
    }

    #[test]
    fn test_draw_main_menu() {
        let mut app = create_test_app();
        let mut terminal = create_test_terminal();
        app.screen = Screen::MainMenu(MainMenuItem::StartGame);

        let result = terminal.draw(|frame| {
            draw(&mut app, frame).expect("drawing should succeed in test");
        });

        assert!(result.is_ok(), "drawing main menu should succeed");
    }

    #[test]
    fn test_draw_options_menu() {
        let mut app = create_test_app();
        let mut terminal = create_test_terminal();
        app.screen = Screen::OptionsMenu(OptionsMenuItem::Map);

        let result = terminal.draw(|frame| {
            draw(&mut app, frame).expect("drawing should succeed in test");
        });

        assert!(result.is_ok(), "drawing options menu should succeed");
    }

    #[test]
    fn test_draw_map_menu() {
        let mut app = create_test_app();
        let mut terminal = create_test_terminal();
        app.screen = Screen::MapMenu;
        app.maps = vec![create_test_map()];
        app.viewport_map = app.maps.first().cloned();

        let result = terminal.draw(|frame| {
            draw(&mut app, frame).expect("drawing should succeed in test");
        });

        assert!(result.is_ok(), "drawing map menu should succeed");
    }

    #[test]
    fn test_draw_in_game() {
        let mut app = create_test_app();
        let mut terminal = create_test_terminal();
        app.screen = Screen::InGame;
        app.map = create_test_map();

        let result = terminal.draw(|frame| {
            draw(&mut app, frame).expect("drawing should succeed in test");
        });

        assert!(result.is_ok(), "drawing in-game screen should succeed");
    }

    #[test]
    fn test_clear_function() {
        let mut terminal = create_test_terminal();

        let result = terminal.draw(|frame| {
            clear(frame);
        });

        assert!(result.is_ok(), "clearing screen should succeed");
    }

    #[test]
    fn test_init_menu_main_menu() {
        let mut terminal = create_test_terminal();

        let result = terminal.draw(|frame| {
            let layout = init_menu(frame, MenuType::MainMenu(3));
            assert_eq!(layout.len(), 3, "main menu should have 3 items");
        });

        assert!(result.is_ok(), "initializing main menu should succeed");
    }

    #[test]
    fn test_init_menu_options_menu() {
        let mut terminal = create_test_terminal();

        let result = terminal.draw(|frame| {
            let layout = init_menu(frame, MenuType::OptionsMenu(2));
            assert_eq!(layout.len(), 2, "options menu should have 2 items");
        });

        assert!(result.is_ok(), "initializing options menu should succeed");
    }

    #[test]
    fn test_main_menu_start_game_selected() {
        let mut terminal = create_test_terminal();

        let result = terminal.draw(|frame| {
            main_menu(frame, MainMenuItem::StartGame);
        });

        assert!(
            result.is_ok(),
            "rendering main menu with start game selected should succeed"
        );
    }

    #[test]
    fn test_main_menu_options_selected() {
        let mut terminal = create_test_terminal();

        let result = terminal.draw(|frame| {
            main_menu(frame, MainMenuItem::Options);
        });

        assert!(
            result.is_ok(),
            "rendering main menu with options selected should succeed"
        );
    }

    #[test]
    fn test_main_menu_quit_selected() {
        let mut terminal = create_test_terminal();

        let result = terminal.draw(|frame| {
            main_menu(frame, MainMenuItem::Quit);
        });

        assert!(
            result.is_ok(),
            "rendering main menu with quit selected should succeed"
        );
    }

    #[test]
    fn test_options_menu_map_selected() {
        let mut terminal = create_test_terminal();

        let result = terminal.draw(|frame| {
            options_menu(frame, OptionsMenuItem::Map);
        });

        assert!(
            result.is_ok(),
            "rendering options menu with map selected should succeed"
        );
    }

    #[test]
    fn test_options_menu_back_selected() {
        let mut terminal = create_test_terminal();

        let result = terminal.draw(|frame| {
            options_menu(frame, OptionsMenuItem::Back);
        });

        assert!(
            result.is_ok(),
            "rendering options menu with back selected should succeed"
        );
    }

    #[test]
    fn test_map_menu_with_maps() {
        let mut app = create_test_app();
        let mut terminal = create_test_terminal();

        app.maps = vec![
            create_test_map(),
            Map {
                key: "second_map".to_owned(),
                data: vec!["222".to_owned(), "213".to_owned(), "222".to_owned()],
            },
        ];
        app.viewport_map = app.maps.first().cloned();

        let result = terminal.draw(|frame| {
            map_menu(&mut app, frame).expect("map menu should render successfully");
        });

        assert!(
            result.is_ok(),
            "rendering map menu with maps should succeed"
        );
    }

    #[test]
    fn test_map_menu_empty_viewport_map_error() {
        let mut app = create_test_app();
        let mut terminal = create_test_terminal();

        app.maps = vec![create_test_map()];
        app.viewport_map = None; // This should cause an error

        let result = terminal.draw(|frame| {
            let map_result = map_menu(&mut app, frame);
            assert!(
                map_result.is_err(),
                "map menu should fail with empty viewport_map"
            );
        });

        assert!(
            result.is_ok(),
            "terminal drawing should succeed even if map_menu fails"
        );
    }

    #[test]
    fn test_in_game_with_valid_map() {
        let mut app = create_test_app();
        let mut terminal = create_test_terminal();

        app.map = create_test_map();
        app.animation_manager = AnimationManager::new();

        let result = terminal.draw(|frame| {
            in_game(&mut app, frame).expect("in-game should render successfully");
        });

        assert!(
            result.is_ok(),
            "rendering in-game with valid map should succeed"
        );
    }

    #[test]
    fn test_in_game_no_entry_point_error() {
        let mut app = create_test_app();
        let mut terminal = create_test_terminal();

        // Map without entry point (no '1' character)
        app.map = Map {
            key: "invalid_map".to_owned(),
            data: vec![
                "222222".to_owned(),
                "200002".to_owned(),
                "222222".to_owned(),
            ],
        };
        app.animation_manager = AnimationManager::new();

        let result = terminal.draw(|frame| {
            let game_result = in_game(&mut app, frame);
            assert!(
                game_result.is_err(),
                "in-game should fail without entry point"
            );
        });

        assert!(
            result.is_ok(),
            "terminal drawing should succeed even if in_game fails"
        );
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
}
