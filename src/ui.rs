use tui::backend::CrosstermBackend;
use tui::Frame;

use tui::layout::{Constraint, Direction, Layout, Rect};

use tui::widgets::ListState;

use crossterm::event::KeyCode;
use std::io::Stdout;

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
    SubWindowInputs,
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
