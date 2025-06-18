//! This module contains the main application logic for the game.

// TODO modularize the code in the entire library

#![expect(
    clippy::cargo_common_metadata,
    reason = "Temporary allow during development."
)]

use std::{
    ffi::OsString,
    fs,
    rc::Rc,
    sync::LazyLock,
    time::{Duration, Instant},
};

use color_eyre::eyre::{OptionExt as _, Result};
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Style},
    symbols::{Marker, DOT},
    text::Line,
    widgets::{
        canvas::{Canvas, Points},
        Block, BorderType, Borders, Clear,
    },
    DefaultTerminal, Frame,
};

/// Animation frame delay in milliseconds.
///
/// This constant controls the timing between animation frames in the pathfinding visualization.
/// A lower value results in faster animation, while a higher value slows down the animation
/// to make it easier to follow the algorithm's progress.
const ANIMATION_FRAME_DELAY_MS: u64 = 200;

/// Animation step types for pathfinding visualization.
///
/// This enumeration represents the different types of steps that can occur during the animated
/// pathfinding visualization, allowing for proper rendering of both forward exploration and
/// backtracking behavior.
#[derive(Debug, Clone)]
enum AnimationStep {
    /// Add a coordinate to the current path visualization.
    ///
    /// This variant represents moving forward in the pathfinding algorithm by adding a new
    /// coordinate to the currently displayed path.
    Add(usize, usize),
    /// Remove a coordinate from the current path visualization.
    ///
    /// This variant represents backtracking in the pathfinding algorithm by removing a coordinate
    /// from the currently displayed path.
    Remove(usize, usize),
}

/// Application state container for the labyrinth game.
///
/// This structure holds the state of the application, which is to say the structure from which
/// Ratatui will render the game and Crossterm events will help writing to.
pub struct App {
    /// Application exit flag.
    ///
    /// This field indicates whether the application should exit. It is set to `true` when the user
    /// wants to quit the game but it starts off `false`.
    exit: bool,
    /// Current screen being displayed to the user.
    ///
    /// This field holds the current screen of the game. It is used to determine which screen to
    /// render and what actions to take based on user input.
    screen: Screen,
    /// Currently active labyrinth map.
    ///
    /// This field holds the current map of the game. It is used to render the labyrinth and solve
    /// it. The custom type always holds a map, either the default one or one loaded and selected by
    /// the user.
    map: Map,
    /// Collection of all available labyrinth maps.
    ///
    /// This field holds information about all the labyrinth maps in the current working directory.
    /// It consists of a key extracted straight from the filesystem and a vector with the contents
    /// of the map as string-rows, stored as custom types within an ordered collection.
    maps: Vec<Map>,
    /// Map currently selected in the viewport.
    ///
    /// This field holds the map that is currently selected in the viewport by the user cursor. This
    /// means the currently selected model in the maps menu.
    viewport_map: Option<Map>,
    /// Scrolling offset for the map list viewport.
    ///
    /// This field holds the offset by which to scroll the sliding window into the
    /// [`maps`](App::maps) vector in the maps menu's viewport.
    viewport_offset: usize,
    /// Height of the map list rendering area.
    ///
    /// This field holds the height of the area in which the list of maps are being rendered as a
    /// measure of terminal cells during the last redraw of the on-screen frame.
    viewport_height: usize,
    /// Animation steps recorded during pathfinding.
    ///
    /// This field stores a sequence of animation steps that represent the pathfinding algorithm's
    /// traversal process, including forward moves and backtracking. Each step contains the
    /// coordinates to be drawn or removed from the visualization.
    animation_steps: Vec<AnimationStep>,
    /// Current step in the animation sequence.
    ///
    /// This field tracks the current position in the [`animation_steps`](App::animation_steps)
    /// vector to determine which steps have been rendered and which are still pending.
    animation_index: usize,
    /// Timestamp of the last animation frame update.
    ///
    /// This field stores the time when the animation was last updated, used to control the timing
    /// between animation frames for smooth visualization.
    last_animation_time: Instant,
    /// Current set of coordinates being displayed in the animation.
    ///
    /// This field maintains the currently visible path coordinates during animation, allowing for
    /// proper backtracking visualization by removing coordinates when needed.
    current_animation_path: Vec<(usize, usize)>,
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
    fn new() -> Self {
        Self {
            exit: false,
            screen: Screen::MainMenu(MainMenuItem::StartGame),
            map: Map::default(),
            maps: Vec::new(),
            viewport_map: None,
            viewport_offset: 0,
            viewport_height: 0,
            animation_steps: Vec::new(),
            animation_index: 0,
            last_animation_time: Instant::now(),
            current_animation_path: Vec::new(),
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
                self.draw(frame)
                    .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))
            })?;
            self.handle_events()?;
        }

        Ok(())
    }

    /// Updates the application UI based on the persistent state.
    ///
    /// This function renders different screens based on the current state stored in the [`App`]
    /// structure, dispatching to the appropriate rendering function for each screen type.
    ///
    /// # Errors
    ///
    /// This function may return errors from drawing operations or data conversion failures.
    fn draw(&mut self, frame: &mut Frame) -> Result<()> {
        match &self.screen {
            Screen::MainMenu(item) => Self::main_menu(frame, *item),
            Screen::OptionsMenu(item) => Self::options_menu(frame, *item),
            Screen::InGame => self.in_game(frame)?,
            Screen::MapMenu => self.map_menu(frame)?,
        }

        Ok(())
    }

    /// Handles input events and updates the application state accordingly.
    ///
    /// This function polls for keyboard events and dispatches them to the appropriate handler
    /// functions based on the key pressed. It uses a timeout to avoid blocking the UI.
    fn handle_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => self.exit = true,
                    KeyCode::Char('j') => self.handle_j_events()?,
                    KeyCode::Char('k') => self.handle_k_events()?,
                    KeyCode::Char('l') => self.handle_l_events()?,
                    KeyCode::Char('h') => self.handle_h_events(),
                    _ => {}
                }
            }
        }

        // Update animation if in-game
        if matches!(self.screen, Screen::InGame) {
            self.update_animation();
        }

        Ok(())
    }

    /// Clears the terminal screen by rendering a [`Clear`] widget.
    ///
    /// This function renders a clear widget over the entire area of the frame to prepare for
    /// rendering new content without artifacts from previous buffers rendered on the same frame.
    fn clear(frame: &mut Frame) {
        let clear = Clear;
        frame.render_widget(clear, frame.area());
    }

    /// Renders the generic layout structure for the main and options menus.
    ///
    /// This function creates the common layout and block structure used by both main and options
    /// menus. The generic part includes the centered positioning and border styling, while the
    /// specific menu content is handled by the caller using the [`MenuType`] parameter.
    #[expect(
        clippy::indexing_slicing,
        reason = "The collection is created in-place with few, known elements; there is no risk of bad indexing."
    )]
    fn init_menu(frame: &mut Frame, menu: MenuType) -> Rc<[Rect]> {
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
    /// This function displays the main menu with options for "Start Game", "Options", and "Quit".
    /// It highlights the currently selected option and provides visual feedback for user
    /// navigation.
    #[expect(
        clippy::indexing_slicing,
        reason = "The collection is created in-place with few, known elements; there is no risk of bad indexing."
    )]
    #[expect(
        clippy::missing_asserts_for_indexing,
        reason = "The collection is created in-place with few, known elements; there is no risk of bad indexing."
    )]
    fn main_menu(frame: &mut Frame, item: MainMenuItem) {
        Self::clear(frame);

        let inner_layout = Self::init_menu(frame, MenuType::MainMenu(3));

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
    fn options_menu(frame: &mut Frame, item: OptionsMenuItem) {
        Self::clear(frame);

        let inner_layout = Self::init_menu(frame, MenuType::OptionsMenu(2));

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
    /// This function displays a viewport containing all loadable maze maps from the current
    /// directory. It provides scrolling functionality and visual indicators for the currently
    /// selected map and the map that's actively being used.
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
    fn map_menu(&mut self, frame: &mut Frame) -> Result<()> {
        Self::clear(frame);

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

        self.viewport_height = inner_space.height.into();

        let inner_layout =
            Layout::horizontal([Constraint::Percentage(5), Constraint::Percentage(100)])
                .split(inner_space);
        let inner_selector = Layout::vertical(vec![Constraint::Max(1); inner_space.height.into()])
            .split(inner_layout[0]);
        let inner_list = Layout::vertical(vec![Constraint::Max(1); inner_space.height.into()])
            .split(inner_layout[1]);

        let mut viewport_maps: Vec<&Map> = self.maps.iter().skip(self.viewport_offset).collect();
        viewport_maps.truncate(inner_space.height.into());

        let content_style = Style::default().fg(Color::Green);
        let active_content_style = Style::default().fg(Color::White).bg(Color::Green);

        for (idx, map) in viewport_maps.into_iter().enumerate() {
            let viewport_map = self
                .viewport_map
                .clone()
                .ok_or_eyre("failed to retrieve cursor-selected map")?;

            let (selector, entry) = if *map == viewport_map {
                (
                    {
                        if *map == self.map {
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
                        if *map == self.map {
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

    /// Transforms maze coordinates to screen coordinates for canvas rendering.
    ///
    /// This function converts maze coordinates (col, row) to screen coordinates (x, y) using the
    /// standard transformation formulas: coordinate[i] = (n - 1) / 2 - i for rows (ascending order)
    /// and coordinate[i] = i - (n - 1) / 2 for columns (descending order).
    ///
    /// # Errors
    ///
    /// This function may return errors from coordinate conversion operations.
    fn transform_maze_to_screen_coords(
        &self,
        maze_coords: &[(usize, usize)],
    ) -> Result<Vec<(f64, f64)>> {
        let rows_n = f64::from(u16::try_from(self.map.data.len())?);
        let cols_n = f64::from(u16::try_from(
            self.map
                .data
                .first()
                .ok_or_eyre("failed to retrieve first element of selected map")?
                .len(),
        )?);

        maze_coords
            .iter()
            .map(|&(col, row)| {
                // Row transformation: coordinate[i] = (n - 1) / 2 - i
                let screen_y = (rows_n - 1.) / 2. - f64::from(u16::try_from(row)?);

                // Column transformation: coordinate[i] = i - (n - 1) / 2
                let screen_x = f64::from(u16::try_from(col)?) - (cols_n - 1.) / 2.;

                Ok((screen_x, screen_y))
            })
            .collect()
    }

    /// Renders the in-game screen with maze visualization and pathfinding solution.
    ///
    /// This function displays the currently selected labyrinth and runs the pathfinding algorithm
    /// to show the solution. It renders both the maze walls and the computed paths using [`Canvas`]
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
    fn in_game(&mut self, frame: &mut Frame) -> Result<()> {
        Self::clear(frame);

        // Initialize animation steps if not already done
        if self.animation_steps.is_empty() {
            // Find the maze entry point (marked with '1')
            let entry_point = self
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
            Self::record_animation_steps(
                &self.map.data,
                entry_point,
                &mut initial_path,
                &mut self.animation_steps,
            );

            self.animation_index = 0;
            self.current_animation_path.clear();
            self.last_animation_time = Instant::now();
        }

        let maze_rows = self.map.data.len();
        let maze_columns = self
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
        for (row_idx, row) in self.map.data.iter().enumerate() {
            for (col_idx, cell) in row.bytes().enumerate() {
                if cell == b'2' {
                    wall_coords.push((col_idx, row_idx));
                }
            }
        }
        let wall_screen_coords = self.transform_maze_to_screen_coords(&wall_coords)?;
        let animation_screen_coords =
            self.transform_maze_to_screen_coords(&self.current_animation_path)?;

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

    /// Records animation steps during pathfinding for later visualization.
    ///
    /// This method performs depth-first search to explore the maze and records each step of the
    /// algorithm (forward moves and backtracking) for animated playback. It captures the exact
    /// sequence of the pathfinding algorithm's exploration from the entry point through the maze.
    fn record_animation_steps(
        map_data: &[String],
        start: (usize, usize),
        current_path: &mut Vec<(usize, usize)>,
        animation_steps: &mut Vec<AnimationStep>,
    ) {
        // Record adding current position to path
        current_path.push(start);
        animation_steps.push(AnimationStep::Add(start.0, start.1));

        // Check if we've reached an exit point ('4')
        if let Some(row) = map_data.get(start.1) {
            if let Some(cell) = row.as_bytes().get(start.0) {
                if *cell == b'4' {
                    // Found exit - record removing position during backtrack
                    let _ = current_path.pop();
                    animation_steps.push(AnimationStep::Remove(start.0, start.1));
                    return;
                }
            }
        }

        // Explore all four directions (north, south, east, west)
        let directions = [(0_i32, -1_i32), (0, 1), (1, 0), (-1, 0)];

        for (dx, dy) in directions {
            // Calculate neighbor coordinates with proper bounds checking
            let Some(new_x) = start.0.checked_add_signed(dx as isize) else {
                continue;
            };
            let Some(new_y) = start.1.checked_add_signed(dy as isize) else {
                continue;
            };

            let new_pos = (new_x, new_y);

            // Skip if already visited in current path
            if current_path.contains(&new_pos) {
                continue;
            }

            // Check if position is valid and walkable
            if let Some(row) = map_data.get(new_pos.1) {
                if let Some(cell) = row.chars().nth(new_pos.0) {
                    // Only explore walkable cells ('3') or exit ('4')
                    if matches!(cell, '3' | '4') {
                        // Recursively explore from this position
                        Self::record_animation_steps(
                            map_data,
                            new_pos,
                            current_path,
                            animation_steps,
                        );
                    }
                }
            }
        }

        // Record removing position during backtrack
        let _ = current_path.pop();
        animation_steps.push(AnimationStep::Remove(start.0, start.1));
    }

    /// Scans the current directory for .labmap files and loads them.
    ///
    /// This function searches for files with the .labmap extension in the current working
    /// directory, validates their format, and adds them to the maps collection for user selection.
    /// It skips invalid files and continues processing valid ones.
    fn fetch_files(&mut self) -> Result<()> {
        for file in fs::read_dir(".")? {
            match file {
                Ok(file)
                    if !file.file_type()?.is_dir()
                        && file
                            .file_name()
                            .to_str()
                            .ok_or_eyre("failed to convert osstring to string slice")?
                            .ends_with(".labmap") =>
                {
                    let contents = fs::read_to_string(file.path())?;

                    if Self::parse_file_contents(contents.trim()) {
                        self.maps.push(Map::new(file.file_name(), &contents)?);
                    }
                }
                Err(err) => return Err(err.into()),
                _ => {}
            }
        }

        Ok(())
    }

    /// Validates the format and content of labyrinth map files.
    ///
    /// This function performs validation to ensure the maze format follows the specification:
    /// - Contains only valid characters (1-4)
    /// - Has consistent row lengths
    /// - Has exactly one entry point (1)
    /// - Is completely surrounded by walls (2s) except for exit points on the edges
    fn parse_file_contents(input: &str) -> bool {
        let lines: Vec<&str> = input.lines().collect();

        // Must have at least 3x3 to form a proper walled maze
        if lines.len() < 3 {
            return false;
        }

        let mut entry_point_counter = 0;
        let Some(first_line) = lines.first() else {
            return false;
        };
        let expected_width = first_line.len();

        // Must have at least 3 columns to form proper walls
        if expected_width < 3 {
            return false;
        }

        // Validate each line
        for line in &lines {
            // Check consistent row lengths
            if line.len() != expected_width {
                return false;
            }

            // Check valid characters only
            if !line
                .bytes()
                .all(|byte| matches!(byte, b'1' | b'2' | b'3' | b'4'))
            {
                return false;
            }

            // Count entry points
            for byte in line.bytes() {
                if byte == b'1' {
                    entry_point_counter += 1;
                }
            }

            // Too many entry points
            if entry_point_counter > 1 {
                return false;
            }
        }

        // Must have exactly one entry point
        if entry_point_counter != 1 {
            return false;
        }

        let last_row_idx = lines.len() - 1;
        let last_col_idx = expected_width - 1;

        // Check boundary walls and validate maze structure in a single pass
        for (row_idx, line) in lines.iter().enumerate() {
            for (col_idx, byte) in line.bytes().enumerate() {
                let is_edge = row_idx == 0
                    || row_idx == last_row_idx
                    || col_idx == 0
                    || col_idx == last_col_idx;

                if is_edge {
                    // On edges: only walls (2) or exit points (4) allowed
                    if !matches!(byte, b'2' | b'4') {
                        return false;
                    }
                } else {
                    // Interior: exit points (4) not allowed
                    if byte == b'4' {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Handles 'j' key press events for downward navigation.
    ///
    /// This function processes the 'j' key press which is used for moving down in menus and lists.
    /// The behavior varies depending on the current screen, handling menu navigation and viewport
    /// scrolling appropriately.
    fn handle_j_events(&mut self) -> Result<()> {
        match self.screen {
            Screen::MainMenu(MainMenuItem::StartGame) => {
                self.screen = Screen::MainMenu(MainMenuItem::Options);
            }
            Screen::MainMenu(MainMenuItem::Options) => {
                self.screen = Screen::MainMenu(MainMenuItem::Quit);
            }
            Screen::OptionsMenu(OptionsMenuItem::Map) => {
                self.screen = Screen::OptionsMenu(OptionsMenuItem::Back);
            }
            Screen::MapMenu => {
                let viewport_map = self
                    .viewport_map
                    .clone()
                    .ok_or_eyre("failed to retrieve cursor-selected map")?;

                if viewport_map
                    == self
                        .maps
                        .iter()
                        .skip(self.viewport_offset)
                        .take(self.viewport_height)
                        .next_back()
                        .ok_or_eyre("no last element in viewport maps")?
                        .clone()
                    && viewport_map
                        != self
                            .maps
                            .last()
                            .ok_or_eyre("failed to retrieve last map")?
                            .clone()
                {
                    self.viewport_offset += 1;
                }

                let mut index = 0;
                for (idx, map) in self.maps.iter().enumerate() {
                    if viewport_map == *map {
                        index = idx;
                        break;
                    }
                }
                match self.maps.get(index + 1) {
                    None => {}
                    Some(element) => {
                        self.viewport_map = Some(element.clone());
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
    fn handle_k_events(&mut self) -> Result<()> {
        match self.screen {
            Screen::MainMenu(MainMenuItem::Quit) => {
                self.screen = Screen::MainMenu(MainMenuItem::Options);
            }
            Screen::MainMenu(MainMenuItem::Options) => {
                self.screen = Screen::MainMenu(MainMenuItem::StartGame);
            }
            Screen::OptionsMenu(OptionsMenuItem::Back) => {
                self.screen = Screen::OptionsMenu(OptionsMenuItem::Map);
            }
            Screen::MapMenu => {
                let viewport_map = self
                    .viewport_map
                    .clone()
                    .ok_or_eyre("failed to retrieve cursor-selected map")?;

                if viewport_map
                    == self
                        .maps
                        .iter()
                        .skip(self.viewport_offset)
                        .take(self.viewport_height)
                        .cloned()
                        .collect::<Vec<Map>>()
                        .first()
                        .ok_or_eyre("no first element in viewport maps")?
                        .clone()
                    && viewport_map
                        != self
                            .maps
                            .first()
                            .ok_or_eyre("failed to retrieve first map")?
                            .clone()
                {
                    self.viewport_offset -= 1;
                }

                let mut index = 0;
                for (idx, map) in self.maps.iter().enumerate() {
                    if viewport_map == *map {
                        index = idx;
                        break;
                    }
                }
                if let Some(element) = self.maps.get(index.saturating_sub(1)) {
                    self.viewport_map = Some(element.clone());
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
    fn handle_l_events(&mut self) -> Result<()> {
        match self.screen {
            Screen::MainMenu(MainMenuItem::StartGame) => {
                self.screen = Screen::InGame;
            }
            Screen::MainMenu(MainMenuItem::Options) => {
                self.screen = Screen::OptionsMenu(OptionsMenuItem::Map);
            }
            Screen::MainMenu(MainMenuItem::Quit) => {
                self.exit = true;
            }
            Screen::OptionsMenu(OptionsMenuItem::Map) => {
                self.screen = Screen::MapMenu;

                let first = Map::default();
                self.maps.clear();
                self.maps.push(first.clone());
                self.fetch_files()?;
                self.viewport_map = Some(first);
                self.viewport_offset = 0;
            }
            Screen::OptionsMenu(OptionsMenuItem::Back) => {
                self.screen = Screen::MainMenu(MainMenuItem::StartGame);
            }
            Screen::MapMenu => {
                self.map = self
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
    fn handle_h_events(&mut self) {
        match self.screen {
            Screen::InGame => {
                // Reset animation state and return to main menu
                self.animation_steps.clear();
                self.animation_index = 0;
                self.current_animation_path.clear();
                self.screen = Screen::MainMenu(MainMenuItem::StartGame);
            }
            Screen::MapMenu => {
                self.screen = Screen::OptionsMenu(OptionsMenuItem::Map);
            }
            _ => {}
        }
    }

    /// Updates the animation state based on timing and current progress.
    ///
    /// This method advances the animation by processing the next step in the animation sequence
    /// when enough time has passed. It handles both adding and removing coordinates from the
    /// current animation path to show the pathfinding exploration and backtracking.
    fn update_animation(&mut self) {
        // Check if enough time has passed for the next animation frame
        if self.last_animation_time.elapsed() >= Duration::from_millis(ANIMATION_FRAME_DELAY_MS) {
            self.last_animation_time = Instant::now();

            if self.animation_index < self.animation_steps.len() {
                // Process the next animation step
                if let Some(step) = self.animation_steps.get(self.animation_index) {
                    match step {
                        AnimationStep::Add(x, y) => {
                            self.current_animation_path.push((*x, *y));
                        }
                        AnimationStep::Remove(x, y) => {
                            // Remove the coordinate from current path (backtracking)
                            if let Some(pos) = self
                                .current_animation_path
                                .iter()
                                .position(|&coord| coord == (*x, *y))
                            {
                                let _ = self.current_animation_path.remove(pos);
                            }
                        }
                    }
                }

                self.animation_index += 1;
            } else {
                // Animation complete, restart from beginning
                self.animation_index = 0;
                self.current_animation_path.clear();
            }
        }
    }
}

/// Enumeration of available application screens.
///
/// This enumeration holds information about the current screen of the game. This is used to
/// determine which screen to render and what actions to take based on user input.
enum Screen {
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
enum MainMenuItem {
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
enum OptionsMenuItem {
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
enum MenuType {
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
    const fn repr(&self) -> &str {
        match self {
            Self::MainMenu(_) => "Main Menu",
            Self::OptionsMenu(_) => "Options Menu",
        }
    }

    /// Returns the numeric value stored by the menu type variant.
    ///
    /// This function provides access to the number of menu items for layout calculations, allowing
    /// the UI to properly size the menu containers.
    const fn value(&self) -> u8 {
        match self {
            Self::MainMenu(value) => *value,
            Self::OptionsMenu(value) => *value,
        }
    }
}

/// Labyrinth map data container.
///
/// This structure represents the custom type employed for indexing into files and retrieving the
/// contents of labyrinth maps. It is used within a vector to get a kind of ordered hashmap.
#[derive(Clone, PartialEq, PartialOrd)]
struct Map {
    /// Display name of the map.
    ///
    /// This field represents the key retrieved as a filename without the file extension for the
    /// map.
    key: String,
    /// Map content as rows of strings.
    ///
    /// This field represents the actual map stored as a vector of strings, each string representing
    /// a row in the map.
    data: Vec<String>,
}

impl Default for Map {
    fn default() -> Self {
        Self::new("Default.labmap".into(), *MAP).expect("failed to create default map")
    }
}

impl Map {
    /// Builds a new map from a filename and multiline string content.
    ///
    /// This function parses the provided string data into individual rows and extracts a clean
    /// filename by removing the .labmap extension. It validates the input and returns an error if
    /// the filename processing fails.
    fn new(key: OsString, data: &str) -> Result<Self> {
        let mut vec = Vec::new();
        for line in data.lines() {
            vec.push(line.to_owned());
        }

        let mut file_name = key
            .to_str()
            .ok_or_eyre("failed to convert osstring to string slice")?
            .to_owned();
        file_name.truncate({
            file_name
                .rfind(".labmap")
                .ok_or_eyre("failed to find extension in file name")?
        });

        Ok(Self {
            key: file_name,
            data: vec,
        })
    }
}

/// Default labyrinth map used as fallback.
///
/// This static holds the default map loaded in both the main game and the map menu.
static MAP: LazyLock<&str> = LazyLock::new(|| {
    "\
2222222222222222222222222222222
2133333333222223333332222223332
2232222223332223232232322223232
2233333223232223232232322223232
2232323223232223232232322222232
2232323223333333232233333333232
2232323222222222232222222222232
2232323333333332233333333332232
2232222222222232222222222232232
2232333333322233333322332232232
2232322232322222232322232232232
2232322232333332232322232232232
2232322232222232232322233332232
2232322233332232232322232232232
2232322222222232232322232232232
2232333333333232232322232232232
2232222222222232232322232232232
2233333332222232232322232232232
2222222232222232232322232232232
2333333333333332232222232233334
2222222222222222222222222222222"
});
