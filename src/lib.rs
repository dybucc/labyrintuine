//! This module contains the main application logic for the game.

#![expect(
    clippy::cargo_common_metadata,
    reason = "Temporary allow during development."
)]

use std::{ffi::OsString, fs, rc::Rc, sync::LazyLock, time::Duration};

use color_eyre::eyre::{OptionExt as _, Result};
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Style},
    symbols::DOT,
    text::Line,
    widgets::{Block, BorderType, Clear},
    DefaultTerminal, Frame,
};

/// This structure holds the state of the application, which is to say the the structure from which
/// Ratatui will render the game and Crossterm events will help writing to.
pub struct App {
    /// This field indicates whether the application should exit. It is set to `true` when the user
    /// wants to quit the game but it starts `false`.
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
    /// This field holds the offset by which to scroll the sliding window into the [`maps`] vector
    /// in the maps menu's viewport.
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
    /// application state. The loop continues until the [`struct@App::field@exit`] field is set to
    /// `true`, after which the function returns to the call site and ratatui restores the state of
    /// the terminal.
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
            Screen::InGame => todo!(),
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

    /// This function handles scanning for files in the directory in which the binary was executed
    /// and checks if there are any .labmap files to read in labyrinth maps for the game.
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

/// This static holds the default map loaded by default in both the main game and the map menu.
static MAP: LazyLock<&str> = LazyLock::new(|| {
    "\
222222222222222222222
133333333333333333332
222222222222222222234"
});
