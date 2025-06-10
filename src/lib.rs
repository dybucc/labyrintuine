//! This module contains the main application logic for the game.

// TODO change the draw function to be fallible and make some of the functions that draw fallible as
// well so that they don't use expect but instead return a result color_eyre type and errors cascade
// back up
// TODO change the file parsing function to accept mazes that are walled (i.e. are surrounded by 2s)
// and disallow mazes that aren't surrounded by 2s except for the exit points (4s can be on the
// edges)
// TODO implement some generic functionality over the pathfinding algorithm to avoid repeating the
// same logic when exploring possible neighbouring cells

#![expect(
    clippy::cargo_common_metadata,
    reason = "Temporary allow during development."
)]

use std::{cmp::Ordering, ffi::OsString, fs, rc::Rc, sync::LazyLock, time::Duration};

use color_eyre::eyre::{OptionExt as _, Result};
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Style},
    symbols::DOT,
    text::Line,
    widgets::{
        canvas::{Canvas, Points},
        Block, BorderType, Clear,
    },
    DefaultTerminal, Frame,
};

/// This structure holds the state of the application, which is to say the the structure from which
/// Ratatui will render the game and Crossterm events will help writing to.
pub struct App {
    /// This field indicates whether the application should exit. It is set to `true` when the user
    /// wants to quit the game but it starts off `false`.
    exit: bool,
    /// This field holds the current screen of the game. It is used to determine which screen to
    /// render and what actions to take based on user input.
    screen: Screen,
    /// This field holds the current map of the game. It is used to render the labyrinth and solve
    /// it. The custom type always holds a map, either the default one or one loaded and selected by
    /// the user.
    map: Map,
    /// This field holds information about all the labyrinth maps in the current working directory.
    /// It consists of a key extracted straight from the filesystem and a vector with the contents
    /// of the map as string-rows, stored as custom types within an ordered collection.
    maps: Vec<Map>,
    /// This field holds the map that is currently selected in the viewport by the user cursor. This
    /// means the currently selected model in the maps menu.
    viewport_map: Option<Map>,
    /// This field holds the offset by which to scroll the sliding window into the
    /// [`maps`](App::maps) vector in the maps menu's viewport.
    viewport_offset: usize,
    /// This field holds the height of the area in which the list of maps are being rendered as a
    /// measure of terminal cells during the last redraw of the on-screen frame.
    viewport_height: usize,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// This function creates a new instance of the [`App`] structure with safe defaults. A
    /// [`Default`] trait implementation is not used here because the struct may perform a fallible
    /// operation in the future. The [`Default`] trait implementation does use this function,
    /// though.
    fn new() -> Self {
        Self {
            exit: false,
            screen: Screen::MainMenu(MainMenuItem::StartGame),
            map: Map::default(),
            maps: Vec::new(),
            viewport_map: None,
            viewport_offset: 0,
            viewport_height: 0,
        }
    }

    /// This function runs the main loop of the application. It handles user input and updates the
    /// application state. The loop continues until the exit condtion is `true`, after which the
    /// function returns to the call site.
    ///
    /// # Errors
    ///
    /// - [`std::io::Error`]
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            let _ = terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    /// This function updates the application UI based on the persistent state stored in the [`App`]
    /// structure.
    fn draw(&mut self, frame: &mut Frame) {
        match &self.screen {
            Screen::MainMenu(item) => Self::main_menu(frame, *item),
            Screen::OptionsMenu(item) => Self::options_menu(frame, *item),
            Screen::InGame => self.in_game(frame),
            Screen::MapMenu => self.map_menu(frame),
        }
    }

    /// This function handles input events and updates the application state accordingly.
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

        Ok(())
    }

    /// This function clears the terminal screen by rendering a clear widget over the entire area
    /// of the frame.
    fn clear(frame: &mut Frame) {
        let clear = Clear;
        frame.render_widget(clear, frame.area());
    }

    /// This function renders the generic part of the main and options menu. This generic part here
    /// means the layout and block in which the menu gets rendered. Because each menu does contains
    /// different entires, this part is non-generic and is made generic through a type [`MenuType`].
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

    /// This function handles rendering on-creen contents in the main menu.
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

    /// This function handles rendering on-creen contents in the options menu.
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

    /// This function handles rendering on-creen contents in the map menu. This entails rendering a
    /// hover viewport of the list of maps loaded from the current directory.
    #[expect(
        clippy::indexing_slicing,
        reason = "The collection is created in-place with few, known elements; there is no risk of bad indexing."
    )]
    #[expect(
        clippy::missing_asserts_for_indexing,
        reason = "The collection is created in-place with few, known elements; there is no risk of bad indexing."
    )]
    fn map_menu(&mut self, frame: &mut Frame) {
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
            let (selector, entry) = if *map
                == self
                    .viewport_map
                    .clone()
                    .expect("failed to retrieve cursor-selected map")
            {
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
    }

    /// This function renders the in-game screen. This consists of rendering a looping animation of
    /// the currently selected labyrinth being solved.
    #[expect(
        clippy::indexing_slicing,
        reason = "The collection is created in-place with few, known elements; there is no risk of bad indexing."
    )]
    #[expect(
        clippy::too_many_lines,
        reason = "Temporary during development; will refactor later."
    )]
    fn in_game(&self, frame: &mut Frame) {
        Self::clear(frame);

        let maze_rows = self.map.data.len();
        let maze_columns = self
            .map
            .data
            .first()
            .expect("failed to retrieve maze in selected map")
            .len();

        let space = Layout::vertical([
            Constraint::Min(1),
            Constraint::Length(
                maze_rows
                    .try_into()
                    .expect("failed to convert maze_rows to u16"),
            ),
            Constraint::Min(1),
        ])
        .split(frame.area())[1];
        let space = Layout::horizontal([
            Constraint::Min(1),
            Constraint::Length(
                maze_columns
                    .try_into()
                    .expect("failed to convert maze_columns to u16"),
            ),
            Constraint::Min(1),
        ])
        .split(space)[1];

        let maze = Canvas::default()
            .x_bounds([
                (-rounded_div::i32(space.width.into(), 2)).into(),
                (rounded_div::i32(space.width.into(), 2)).into(),
            ])
            .y_bounds([
                (-rounded_div::i32(space.height.into(), 2)).into(),
                (rounded_div::i32(space.height.into(), 2)).into(),
            ])
            .paint(|ctx| {
                let mut output = Vec::new();

                ctx.draw(&Points {
                    coords: {
                        // The formula to create evenly distributed coordinates around the origin is
                        // coordinate[i] = (n - 1) / 2 - i in ascending order
                        // coordinate[i] = i - (n - 1) / 2 in descending order

                        // Rows computation.
                        let mut rows = Vec::new();
                        let n = f64::from(
                            u16::try_from(self.map.data.len()).expect("failed to convert rows"),
                        );
                        for (idx, row) in self.map.data.iter().enumerate() {
                            if row.contains('2') {
                                let idx =
                                    f64::from(u16::try_from(idx).expect("failed to convert rows"));
                                rows.push((n - 1.) / 2. - idx);
                            }
                        }

                        // Columns computation.
                        let mut cols = Vec::new();
                        let n = f64::from(
                            u16::try_from(
                                self.map
                                    .data
                                    .first()
                                    .expect("failed to retrieve first element of selected map")
                                    .len(),
                            )
                            .expect("failed to convert columns"),
                        );
                        for row in &self.map.data {
                            let mut inner = Vec::new();
                            for (idx, col) in row.bytes().enumerate() {
                                if col == b'2' {
                                    let idx = f64::from(
                                        u16::try_from(idx).expect("failed to convert rows"),
                                    );
                                    inner.push(idx - (n - 1.) / 2.);
                                }
                            }
                            cols.push(inner);
                        }

                        for (idx, row) in rows.iter().enumerate() {
                            for col in &cols[idx] {
                                output.push((*col, *row));
                            }
                        }

                        &output
                    },
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
            .paint(|ctx| {
                let mut all_paths_cells: Vec<Vec<(usize, usize)>> = Vec::new();
                let mut marked_cells: Vec<Vec<MapCell>> = self
                    .map
                    .data
                    .iter()
                    .map(|line| {
                        line.bytes()
                            .map(|char| {
                                if matches!(char, b'1' | b'3' | b'4') {
                                    MapCell::Unexplored
                                } else {
                                    MapCell::None
                                }
                            })
                            .collect::<Vec<MapCell>>()
                    })
                    .collect();
                let root = MapFork {
                    cell: None,
                    children: Vec::new(),
                };
                let mut parent;

                loop {
                    parent = root.clone();
                    let mut path_cells = Vec::new();
                    let mut current_cell = self
                        .map
                        .data
                        .iter()
                        .enumerate()
                        .find_map(|(row, line)| {
                            line.bytes()
                                .enumerate()
                                .find_map(|(col, char)| (char == b'1').then_some((col, row)))
                        })
                        .expect("failed to retrieve entry point in map");

                    'inner: loop {
                        path_cells.push(current_cell);
                        *marked_cells
                            .get_mut(current_cell.1)
                            .expect("failed to retrieve current cell row")
                            .get_mut(current_cell.0)
                            .expect("failed to retrieve current cell col") = MapCell::Explored;
                        let mut possible_cells = Vec::new();

                        // Check north (x, y - 1)
                        if let Some(row) = self.map.data.get(current_cell.1.wrapping_sub(1)) {
                            if let Some(col) = row.get(current_cell.0..=current_cell.0) {
                                let iteration_cell = (
                                    row.bytes()
                                        .position(|value| value == col.as_bytes()[0])
                                        .expect("failed to retrieve new col"),
                                    self.map
                                        .data
                                        .iter()
                                        .position(|value| value == row)
                                        .expect("failed to retrieve new row"),
                                );
                                if matches!(col, "3" | "4") && !path_cells.contains(&iteration_cell)
                                {
                                    possible_cells.push(iteration_cell);
                                }
                            }
                        }

                        // Check south (x, y + 1)
                        if let Some(row) = self.map.data.get(current_cell.1 + 1) {
                            if let Some(col) = row.get(current_cell.0..=current_cell.0) {
                                let iteration_cell = (
                                    row.bytes()
                                        .position(|value| value == col.as_bytes()[0])
                                        .expect("failed to retrieve new col"),
                                    self.map
                                        .data
                                        .iter()
                                        .position(|value| value == row)
                                        .expect("failed to retrieve new row"),
                                );
                                if matches!(col, "3" | "4") && !path_cells.contains(&iteration_cell)
                                {
                                    possible_cells.push(iteration_cell);
                                }
                            }
                        }

                        // Check west (x - 1, y)
                        if let Some(row) = self.map.data.get(current_cell.1) {
                            if let Some(col) =
                                row.get(current_cell.0.wrapping_sub(1)..current_cell.0)
                            {
                                let iteration_cell = (
                                    row.bytes()
                                        .position(|value| value == col.as_bytes()[0])
                                        .expect("failed to retrieve new col"),
                                    self.map
                                        .data
                                        .iter()
                                        .position(|value| value == row)
                                        .expect("failed to retrieve new row"),
                                );
                                if matches!(col, "3" | "4") && !path_cells.contains(&iteration_cell)
                                {
                                    possible_cells.push(iteration_cell);
                                }
                            }
                        }

                        // Check east (x + 1, y)
                        if let Some(row) = self.map.data.get(current_cell.1) {
                            if let Some(col) = row.get((current_cell.0 + 1)..=(current_cell.0 + 1))
                            {
                                let iteration_cell = (
                                    row.bytes()
                                        .position(|value| value == col.as_bytes()[0])
                                        .expect("failed to retrieve new col"),
                                    self.map
                                        .data
                                        .iter()
                                        .position(|value| value == row)
                                        .expect("failed to retrieve new row"),
                                );
                                if matches!(col, "3" | "4") && !path_cells.contains(&iteration_cell)
                                {
                                    possible_cells.push(iteration_cell);
                                }
                            }
                        }

                        match possible_cells.len().cmp(&1) {
                            Ordering::Greater => {
                                let new_fork = MapFork {
                                    cell: Some(current_cell),
                                    children: Vec::new(),
                                };
                                if possible_cells.iter().all(|&cell| {
                                    *marked_cells
                                        .get(cell.1)
                                        .expect("failed to retrieve row")
                                        .get(cell.0)
                                        .expect("failed to retrieve col")
                                        == MapCell::Unexplored
                                }) {
                                    parent.children.push(new_fork.clone());
                                }
                                parent = new_fork;

                                if possible_cells.iter().any(|&cell| {
                                    *marked_cells
                                        .get(cell.1)
                                        .expect("failed to retrieve row")
                                        .get(cell.0)
                                        .expect("faled to retrieve col")
                                        == MapCell::Unexplored
                                }) {
                                    let filtered_cells: Vec<(usize, usize)> = possible_cells
                                        .iter()
                                        .filter(|&&cell| {
                                            *marked_cells
                                                .get(cell.1)
                                                .expect("failed to retrieve row")
                                                .get(cell.0)
                                                .expect("faled to retrieve col")
                                                == MapCell::Unexplored
                                        })
                                        .copied()
                                        .collect();

                                    current_cell = *filtered_cells
                                        .first()
                                        .expect("failed to retrieve unexplored cell");

                                    continue 'inner;
                                }

                                let mut competents = vec![0; parent.children.len()];
                                for (idx, fork) in parent.children.iter().enumerate() {
                                    Self::recursive_exploration(
                                        &mut competents,
                                        &marked_cells,
                                        fork,
                                        idx,
                                    );
                                }
                                let winner = competents.into_iter().max().expect(
                                    "failed to retrieve competent with the most brownie points",
                                );
                                if winner != 0 {
                                    // TODO assign to the current cell.
                                    // At this point, we have the forking point coordinate, the next
                                    // forking point, and a past path that already connected the
                                    // two. One can trust this invariant because at this point, the
                                    // forking point that the current cell is at does not have any
                                    // unexplored cells around it, and the next forking point that
                                    // should be explored is already registered as a child node of
                                    // the current cell from already having gone through it in
                                    // another path during some other iteration. Thus, the current
                                    // cell's new value will be determined by the first cell that
                                    // connected the part of a path that happened to traverse these
                                    // two points. This will be true for any paths containing both
                                    // points.
                                    // Summary: search in the vector containing all paths for a path
                                    // that contains both points. Then take that path, search for
                                    // the current cell's position, and fetch the cell that comes
                                    // right after it in the ordered collection that the path is.
                                    // That will be the new value of the current cell.

                                    continue 'inner;
                                }
                            }
                            Ordering::Equal => {
                                current_cell = *possible_cells
                                    .first()
                                    .expect("failed to retrieve next cell");

                                continue 'inner;
                            }
                            _ => {}
                        }

                        break;
                    }

                    all_paths_cells.push(path_cells);

                    if marked_cells.iter().all(|value| {
                        value
                            .iter()
                            .all(|value| matches!(value, MapCell::Explored | MapCell::None))
                    }) {
                        break;
                    }
                }
            });

        frame.render_widget(maze, space);
        frame.render_widget(solution, space);
    }

    /// This function recursively seeks for free, unexplored cells in the given forking point, by
    /// exploring all child nodes. For each child node, it considers if there are any unexplored
    /// cells around, and if there are, then this adds 'brownie' points to the current iteration
    /// competent. The competent is a single, constant element in the `competents` vector that is
    /// initially called with the first stack frame allocation. This function thus relies on prior
    /// iteration through a vector of competents (which is NOT the `competents` vector, as this only
    /// holds the counter.)
    fn recursive_exploration(
        competents: &mut Vec<i32>,
        marked_cells: &Vec<Vec<MapCell>>,
        current: &MapFork,
        current_competent: usize,
    ) {
        let cell = current
            .cell
            .expect("failed to retrieve non-virtual, non-root, child node in tree");
        let mut check = |pos1: usize, pos2: usize| {
            if let Some(row) = marked_cells.get(pos1) {
                if let Some(col) = row.get(pos2) {
                    if *col == MapCell::Unexplored {
                        *competents
                            .get_mut(current_competent)
                            .expect("failed to retrieve competent") += 1;
                    }
                }
            }
        };
        let positions = [
            (cell.1 - 1, cell.0),
            (cell.1 + 1, cell.0),
            (cell.1, cell.0 - 1),
            (cell.1, cell.0 + 1),
        ];

        for (pos1, pos2) in positions {
            check(pos1, pos2);
        }

        if current.children.is_empty() {
            return;
        }
        for child in &current.children {
            Self::recursive_exploration(competents, marked_cells, child, current_competent);
        }
    }

    /// This function handles scanning for files in the directory in which the binary was executed.
    /// It checks for .labmap files to read in maze maps.
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

    /// This function serves as a way to parse the contents of input files that have been deemed to
    /// be apparently appropiate for processing by the program but whose contents have not yet been
    /// confirmed to work with the format spec for labyrinths. This function makes that last check.
    fn parse_file_contents(input: &str) -> bool {
        let mut entry_point_counter = 0;
        let mut previous_line = "";
        let mut string_vector = Vec::new();

        for (idx, line) in input.lines().enumerate() {
            if line
                .bytes()
                .all(|char| matches!(char, b'1' | b'2' | b'3' | b'4'))
            {
                for char in line.bytes() {
                    if char == b'1' {
                        entry_point_counter += 1;
                    }
                }
                if entry_point_counter > 1 {
                    break;
                }
                if idx > 0 && line.len() != previous_line.len() {
                    break;
                }

                string_vector.push(line);
            } else {
                break;
            }

            previous_line = line;
        }

        if string_vector.len() == input.lines().count() {
            for line in string_vector.iter().filter(|line| {
                *line
                    != string_vector
                        .first()
                        .expect("failed to retrieve first element in final string vector")
                    && *line
                        != string_vector
                            .last()
                            .expect("failed to retrieve last element in final string vector")
            }) {
                for (idx, _) in line.match_indices(['4', '1']) {
                    if idx != 0 && idx != line.len() - 1 {
                        return false;
                    }
                }
            }

            true
        } else {
            false
        }
    }

    /// This function handles events where the user input was a keypress on the 'j' key.
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
                        .last()
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

    /// This function handles events where the user input was a keypress on the 'k' key.
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

    /// This function handles events where the user input was a keypress on the 'l' key.
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

    /// This function handles events where the user input was a keypress on the 'h' key.
    fn handle_h_events(&mut self) {
        if matches!(self.screen, Screen::MapMenu) {
            self.screen = Screen::OptionsMenu(OptionsMenuItem::Map);
        }
    }
}

/// This enumeration holds information about the current screen of the game. This is used to
/// determine which screen to render and what actions to take based on user input.
enum Screen {
    /// This variant represents the main menu screen of the game.
    MainMenu(MainMenuItem),
    /// This variant represents the options menu screen of the game.
    OptionsMenu(OptionsMenuItem),
    /// This variant represents the ingame screen where the labyrinth is displayed and solved.
    InGame,
    /// This variant represents the map menu screen of the game. It contains a list of the maps
    /// available to the user.
    MapMenu,
}

/// This enumeration holds the different items in the main menu. It is used to determine which
/// items can the user select in the main menu.
#[derive(Clone, Copy)]
enum MainMenuItem {
    /// This variant represents the "Start Game" option in the main menu.
    StartGame,
    /// This variant represents the "Options" option in the main menu.
    Options,
    /// This variant represents the "Quit" option in the main menu.
    Quit,
}

/// This enumeration holds the different items in the options menu. It is used to determine which
/// items can the user select in the options menu.
#[derive(Clone, Copy)]
enum OptionsMenuItem {
    /// This variant represents the "Back" option in the options menu.
    Back,
    /// This variant represents the "Map" option in the options menu.
    Map,
}

/// This enumeration holds the different specifics particular to each generic menu type in the
/// application's interface. Generic here means they share enough features to be considered worth
/// joining together part of their functionality.
enum MenuType {
    /// This variant represents the main menu in the game.
    MainMenu(u8),
    /// This variant represents the options menu in the game.
    OptionsMenu(u8),
}

impl MenuType {
    /// This function returns the string representation of the variant of the corresponding
    /// enumeration, for use as part of the specifics of each menu type.
    const fn repr(&self) -> &str {
        match self {
            Self::MainMenu(_) => "Main Menu",
            Self::OptionsMenu(_) => "Options Menu",
        }
    }

    /// This function returns the value stored by each enumeration variant.
    const fn value(&self) -> u8 {
        match self {
            Self::MainMenu(value) => *value,
            Self::OptionsMenu(value) => *value,
        }
    }
}

/// This structure represents the custom type employed for indexing into files and retrieving
/// the contents of labyrinth maps. It is used within a vector to get a kind of ordered hashmap.
#[derive(Clone, PartialEq, PartialOrd)]
struct Map {
    /// This field represents the key retrieved as a filename without the file extension for the
    /// map.
    key: String,
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
    /// This function builds a new map given a file name and a multiline string slice.
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

/// This enumeration serves as a way of mapping elements of a maze map to possible states in which
/// each cell might find itself during the solving process.
#[derive(PartialEq, PartialOrd)]
enum MapCell {
    /// This variant represents the state of already having explored the cell.
    Explored,
    /// This variant represents the state of not yet having explored the cell.
    Unexplored,
    /// This variant represents cells that are neither to be explored nor have been explored. This
    /// is useful for blocker cells alone, which are not meant to be explored.
    None,
}

/// This structure represents cells in the map where multiple paths may be taken (i.e. forks.)
#[derive(Clone)]
struct MapFork {
    /// This field represents the cell in question where the path is divided in two or more paths.
    cell: Option<(usize, usize)>,
    /// This field represents the vector of possible paths to take from the [`cell`](MapFork::cell)
    /// field onwards.
    children: Vec<MapFork>,
}

/// This static holds the default map loaded by default in both the main game and the map menu.
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
