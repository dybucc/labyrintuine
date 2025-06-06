//! This module contains the main application logic for the game.

#![expect(
    clippy::cargo_common_metadata,
    reason = "Temporary allow during development."
)]

use std::{collections::HashMap, ffi::OsString, fs, rc::Rc, time::Duration};

use color_eyre::eyre::{OptionExt as _, Result};
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, BorderType, Clear},
    DefaultTerminal, Frame,
};

/// This structure holds the state of the application, which is to say the the structure from which
/// Ratatui will render the game and Crossterm events will help writing to.
pub struct App<'map> {
    /// This field indicates whether the application should exit. It is set to `true` when the user
    /// wants to quit the game but it starts `false`.
    exit: bool,
    /// This field holds the current screen of the game. It is used to determine which screen to
    /// render and what actions to take based on user input.
    screen: Screen,
    /// This field holds the current map of the game. It is used to render the labyrinth and solve
    /// it. It is an `Option` because the map may be the default map, which is `None`, or it may
    /// be a custom map provided by the user and loaded from disk.
    selected_map: Option<&'map str>,
    /// This field holds information about all the labyrinth maps in the current working directory.
    /// It consists of a key extracted straight from the filesystem and a string with the contents
    /// of the map.
    maps: HashMap<OsString, String>,
}

impl Default for App<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl App<'_> {
    /// This function creates a new instance of the [`App`] structure with safe defaults. A
    /// [`Default`] trait implementation is not used here because the struct may perform a fallible
    /// operation in the future. The [`Default`] trait implementation does use this function,
    /// though.
    fn new() -> Self {
        Self {
            exit: false,
            screen: Screen::MainMenu(MainMenuItem::StartGame),
            selected_map: None,
            maps: HashMap::new(),
        }
    }

    /// This function runs the main loop of the application. It handles user input and updates the
    /// application state. The loop continues until the [`exit`] field is set to `true`, after which
    /// the function returns to the call site and ratatui restores the state of the terminal.
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
    fn draw(&self, frame: &mut Frame) {
        match &self.screen {
            Screen::MainMenu(item) => Self::main_menu(frame, *item),
            Screen::OptionsMenu(item) => Self::options_menu(frame, *item),
            Screen::InGame => todo!(),
            Screen::MapMenu => todo!(),
        }
    }

    /// This function handles input events and updates the application state accordingly.
    fn handle_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('j') => self.handle_j_events(),
                    KeyCode::Char('k') => self.handle_k_events(),
                    KeyCode::Char('l') => self.handle_l_events(),
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
    fn map_menu(&self, frame: &mut Frame) {
        Self::clear(frame);

        // TODO

        let layout = Layout::vertical([Constraint::Length(todo!())]);
    }

    /// This function handles scanning for files in the directory in which the binary was executed
    /// and checks if there are any .labmap files to read in labyrinth maps for the game.
    fn fetch_files(&mut self) -> Result<()> {
        for file in fs::read_dir(".")? {
            match file {
                Ok(file)
                    if file
                        .file_name()
                        .to_str()
                        .ok_or_eyre("failed to convert osstring to string slice")?
                        .ends_with(".labmap") =>
                {
                    let contents = fs::read_to_string(file.path())?;

                    // TODO verify that the contents of the file are correct
                    // this requires specifying a format for the input files, as well as a parser
                    // for that type of files
                    // ideally left to another function

                    let _ = self
                        .maps
                        .insert(file.file_name(), contents)
                        .ok_or_eyre("failed to insert file contents on maps map")?;
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
    fn parse_file_contents(input: &str) {
        // the format spec will consist of two types of characters
        // the first one will define paths that don't have anything in their way
        // the second one will define paths that have a blocker in their way
        // a map should consist of a block that will be parsed line by line as either a square or a
        // rectangle

        todo!()
    }

    /// This function handles events where the user input was a keypress on the j input.
    fn handle_j_events(&mut self) {
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
            _ => {}
        }
    }

    /// This function handles events where the user input was a keypress on the k input.
    fn handle_k_events(&mut self) {
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
            _ => {}
        }
    }

    /// This function handles events where the user input was a keypress on the l input.
    fn handle_l_events(&mut self) {
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
                // fetch the list of maps stored as files in the current directory
            }
            Screen::OptionsMenu(OptionsMenuItem::Back) => {
                self.screen = Screen::MainMenu(MainMenuItem::StartGame);
            }
            _ => {}
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
