use tui::backend::CrosstermBackend;
use std::io::Stdout;
use tui::Frame;
use tui::layout::Rect;
use crossterm::event::KeyCode;
use crate::structure::application::ApplicationState;

pub mod project_s;
pub mod task_s;

pub trait Service {
    fn set_working_directory(&mut self, path: std::path::PathBuf);
    fn display(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, layout: Rect);
    fn handle_input(&mut self, key_code: KeyCode) -> bool;
    fn get_input_possibilities(&self) -> String;
    // We give each tab the possibility to modify what they need without limiting access.
    fn update_application_state(&mut self, application_state: &mut ApplicationState);
}
