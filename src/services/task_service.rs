use std::path::PathBuf;
use crate::services::Service;
use crate::ui::{Drawable, InputReceptor, InputMode};
use tui::layout::Rect;
use tui::Frame;
use tui::backend::CrosstermBackend;
use std::io::Stdout;
use crossterm::event::KeyCode;

struct TaskService {
    working_project: PathBuf
}

impl Service for TaskService {
    fn set_working_directory(&mut self, path: PathBuf) {
        self.working_project = path;
    }
}

impl Drawable for TaskService {
    fn display(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, layout: Rect) {
        unimplemented!()
    }
}

impl InputReceptor for TaskService {
    fn handle_input_key(&mut self, key_code: KeyCode) {
        unimplemented!()
    }

    fn get_controls_description(&self) -> String {
        unimplemented!()
    }

    fn get_input_mode(&self) -> InputMode {
        unimplemented!()
    }
}

