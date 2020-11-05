use serde::Deserialize;
use serde::Serialize;

use std::io;
use tui::backend::CrosstermBackend;
use tui::{Frame, Terminal};

use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::text::Text;
use tui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};

use crate::structure::Project;
use crate::utils;
use crate::utils::{get_projects_in_path, get_working_folder};
use crate::Event::Input;
use crossterm::event::Event::Key;
use crossterm::event::KeyCode;
use serde_json;
use std::io::Stdout;
use std::ops::Add;
use std::path::PathBuf;
use std::ptr::null;
use tui::style::{Color, Style};

#[derive(Default)]
pub struct DisplayList<T> {
    pub(crate) state: ListState,
    pub(crate) array: Vec<T>,
}

pub enum InputMode {
    CommandMode,
    WriteMode,
}

// Seperate:
    // Trait: Renderable
    // Trait: InputReturn // A type that returns information

pub trait Drawable {
    fn display(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, layout: Rect);
    fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_y) / 2),
                    Constraint::Percentage(percent_y),
                    Constraint::Percentage((100 - percent_y) / 2),
                ]
                    .as_ref(),
            )
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_x) / 2),
                    Constraint::Percentage(percent_x),
                    Constraint::Percentage((100 - percent_x) / 2),
                ]
                    .as_ref(),
            )
            .split(popup_layout[1])[1]
    }
}

pub trait InputReceptor {
    fn handle_input_key(&mut self, key_code: KeyCode);
    fn get_controls_description(&self) -> String;
    fn get_input_mode(&self) -> InputMode;
}

pub trait InputReturn {
    fn get_input_data(&self) -> String;
}

pub trait Completable {
    fn is_completed(&self) -> bool;
    fn reset_completion(&mut self);
    fn is_active(&self) -> bool;
    fn set_active(&mut self, new_active: bool);
}


// TODO: popup types:
// - Yes or no popup

impl<T> DisplayList<T> {
    pub(crate) fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i < self.array.len() - 1 {
                    i + 1
                } else {
                    i
                }
            }
            None => 0,
        };
        if self.array.len() > 0 {
            self.state.select(Some(i));
        }
    }
    pub(crate) fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i > 0 {
                    i - 1
                } else {
                    0
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn from(content: Vec<T>) -> DisplayList<T> {
        let content_len = content.len();
        let mut dl = DisplayList {
            state: Default::default(),
            array: content,
        };
        if content_len > 0 {
            dl.state.select(Some(0));
        }
        dl
    }
}

#[derive(Default)]
pub struct PopupMessageWindow {
    description: String,
    is_active: bool,
    is_done: bool,
}

impl PopupMessageWindow {
    pub fn new(popup_message: String) -> PopupMessageWindow {
        PopupMessageWindow{
            description: popup_message,
            is_active: true,
            is_done: false,
        }
    }
}

impl Completable for PopupMessageWindow {
    fn is_completed(&self) -> bool {
        self.is_done
    }

    fn reset_completion(&mut self) {
        self.is_done = false;
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn set_active(&mut self, new_active: bool) {
        self.is_active = new_active;
    }
}

impl Drawable for PopupMessageWindow {
    fn display(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, layout: Rect) {
        let popup_layout = self.centered_rect(50, 25, layout);
        frame.render_widget(Clear, popup_layout);
        let popup_block = Block::default().borders(Borders::ALL);
        frame.render_widget(popup_block, popup_layout);
        let main_popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(10), Constraint::Percentage(90)])
            .split(popup_layout);
        let description_paragraph =
            Paragraph::new(Text::from(self.description.clone())).alignment(Alignment::Center);
        frame.render_widget(description_paragraph, main_popup_layout[0]);
        let block = Block::default().borders(Borders::ALL);
        let ok_message = Paragraph::new(Text::from("Ok"))
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Center)
            .block(block);
        frame.render_widget(ok_message, main_popup_layout[1]);
    }
}

impl InputReceptor for PopupMessageWindow {
    fn handle_input_key(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Enter => {
                self.is_done = true;
            }
            _ => {}
        }
    }

    fn get_controls_description(&self) -> String {
        String::from("Press enter to continue")
    }

    fn get_input_mode(&self) -> InputMode {
        InputMode::CommandMode
    }
}

// PopupBinaryChoice

#[derive(Default)]
pub struct PopupBinaryChoice {
    choice_message: String,
    current_choice: bool,
    is_completed: bool,
    is_active: bool,
}

impl PopupBinaryChoice {
    pub fn new(message: String) -> PopupBinaryChoice {
        PopupBinaryChoice {
            choice_message: message,
            current_choice: false,
            is_completed: false,
            is_active: true,
        }
    }

    pub fn get_choice(&self) -> bool {
        self.current_choice
    }
}

impl InputReceptor for PopupBinaryChoice {
    fn handle_input_key(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Left => self.current_choice = true,
            KeyCode::Right => self.current_choice = false,
            KeyCode::Enter => self.is_completed = true,
            _ => {},
        }
    }

    fn get_controls_description(&self) -> String {
        String::from("<-: Go Left  |  ->: Go Right  |  Enter: Confirm Selection ")
    }

    fn get_input_mode(&self) -> InputMode {
        InputMode::CommandMode
    }
}

impl Completable for PopupBinaryChoice {
    fn is_completed(&self) -> bool {
        self.is_completed
    }

    fn reset_completion(&mut self) {
        self.is_completed = false;
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn set_active(&mut self, new_active: bool) {
        self.is_active = new_active;
    }
}

impl Drawable for PopupBinaryChoice {
    fn display(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, layout: Rect) {
        let popup_layout = self.centered_rect(50, 20, layout);
        frame.render_widget(Clear, popup_layout);
        let main_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(popup_layout);
        let popup_block = Block::default().borders(Borders::ALL);
        frame.render_widget(popup_block, popup_layout);
        let message_paragraph = Paragraph::new(Text::from(self.choice_message.clone()))
            .alignment(Alignment::Center)
            .wrap(Wrap{trim:false});
        frame.render_widget(message_paragraph, main_split[0]);
        let choice_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_split[1]);
        let choice_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick);
        let mut yes_paragraph = Paragraph::new(Text::from("Yes"))
            .alignment(Alignment::Center);
        let mut no_paragraph = Paragraph::new(Text::from("No"))
            .alignment(Alignment::Center);
        if self.current_choice {
            yes_paragraph = yes_paragraph.block(choice_block);
        } else {
            no_paragraph = no_paragraph.block(choice_block);
        }
        frame.render_widget(yes_paragraph, choice_layout[0]);
        frame.render_widget(no_paragraph, choice_layout[1]);
    }
}

// PopupInputWindow ---------------- Todo: refactor into another file (use module folders)

#[derive(Default)]
pub struct PopupInputWindow {
    description: String,
    input_string: String,
    is_active: bool,
    message_input_finished: bool,
}

impl PopupInputWindow {
    pub fn new(popup_description: String) -> PopupInputWindow {
        PopupInputWindow {
            description: popup_description,
            input_string: String::new(),
            is_active: true,
            message_input_finished: false,
        }
    }
}

impl Drawable for PopupInputWindow {
    fn display(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, layout: Rect) {
        let popup_layout = self.centered_rect(50, 25, layout);
        frame.render_widget(Clear, popup_layout);
        let popup_block = Block::default().borders(Borders::ALL);
        frame.render_widget(popup_block, popup_layout);
        let main_popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(10), Constraint::Percentage(90)])
            .split(popup_layout);
        let description_paragraph = Paragraph::new(Text::from(self.get_controls_description()))
            .alignment(Alignment::Center);
        frame.render_widget(description_paragraph, main_popup_layout[0]);
        let input_paragraph = Paragraph::new(Text::from(self.input_string.clone()))
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Center);
        frame.render_widget(input_paragraph, main_popup_layout[1]);
    }
}

impl Completable for PopupInputWindow {
    fn is_completed(&self) -> bool {
        self.message_input_finished
    }

    fn reset_completion(&mut self) {
        self.message_input_finished = false;
    }

    fn is_active(&self) -> bool {
        self.is_active
    }

    fn set_active(&mut self, new_active: bool) {
        self.is_active = new_active;
    }
}

impl InputReturn for PopupInputWindow {
    fn get_input_data(&self) -> String {
        self.input_string.clone()
    }
}

impl InputReceptor for PopupInputWindow {
    fn handle_input_key(&mut self, key_code: KeyCode) {
        match key_code {
            KeyCode::Char(char) => {
                self.input_string.push(char);
            }
            KeyCode::Backspace => {
                if self.input_string.len() > 0 {
                    self.input_string.pop();
                }
            }
            KeyCode::Enter => {
                self.message_input_finished = true;
            }
            KeyCode::Esc => {
                self.set_active(false);
            }
            _ => {}
        };
    }

    fn get_controls_description(&self) -> String {
        String::from(self.description.clone())
    }

    fn get_input_mode(&self) -> InputMode {
        InputMode::CommandMode
    }
}

