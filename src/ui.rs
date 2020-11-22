use tui::backend::CrosstermBackend;
use tui::Frame;

use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::text::Text;
use tui::widgets::{Block, BorderType, Borders, Clear, ListState, Paragraph, Wrap};

use crossterm::event::KeyCode;
use std::io::Stdout;
use tui::style::{Color, Style};

#[derive(Default)]
pub struct DisplayList<T> {
    pub(crate) state: ListState,
    pub(crate) array: Vec<T>,
}

#[derive(SmartDefault)]
pub enum InputMode {
    #[default]
    CommandMode,
    WriteMode,
}

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

impl<T> DisplayList<T> {
    pub(crate) fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if self.array.len() > 0 {
                    if i < self.array.len() - 1 {
                        i + 1
                    } else {
                        i
                    }
                } else {
                    0
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
        if self.array.len() > 0 {
            self.state.select(Some(i));
        }
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

// PopupMessageWindow

impl PopupMessageWindow {
    pub fn new(popup_message: String) -> PopupMessageWindow {
        PopupMessageWindow {
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
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
            .split(popup_layout);
        let block = Block::default().borders(Borders::ALL);
        let description_paragraph = Paragraph::new(Text::from(self.description.clone()))
            .alignment(Alignment::Center)
            .block(block)
            .wrap(Wrap { trim: false });
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
            _ => {}
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
        let message_block = Block::default().borders(Borders::ALL);
        let message_paragraph = Paragraph::new(Text::from(self.choice_message.clone()))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false })
            .block(message_block);
        frame.render_widget(message_paragraph, main_split[0]);
        let choice_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(main_split[1]);
        let normal_block = Block::default().borders(Borders::ALL);
        let choice_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Thick)
            .style(Style::default().fg(Color::Red));
        let mut yes_paragraph = Paragraph::new(Text::from("Yes")).alignment(Alignment::Center);
        let mut no_paragraph = Paragraph::new(Text::from("No")).alignment(Alignment::Center);
        if self.current_choice {
            yes_paragraph = yes_paragraph.block(choice_block);
            no_paragraph = no_paragraph.block(normal_block);
        } else {
            no_paragraph = no_paragraph.block(choice_block);
            yes_paragraph = yes_paragraph.block(normal_block);
        }
        frame.render_widget(yes_paragraph, choice_layout[0]);
        frame.render_widget(no_paragraph, choice_layout[1]);
    }
}

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

    pub fn set_input_string(&mut self, new_input_string: String) {
        self.input_string = new_input_string;
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
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(popup_layout);
        let description_paragraph =
            Paragraph::new(Text::from(self.description.clone())).alignment(Alignment::Center);
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
        String::from("esc - Cancel | Enter - Confirm entry")
    }

    fn get_input_mode(&self) -> InputMode {
        InputMode::CommandMode
    }
}
