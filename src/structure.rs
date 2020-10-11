use serde::Deserialize;
use serde::Serialize;

use std::io;
use tui::backend::CrosstermBackend;
use tui::{Frame, Terminal};

use tui::layout::{Constraint, Direction, Layout};
use tui::text::Text;
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use crate::ui::{ProjectWindow, Window};
use crate::utils;

use std::io::Stdout;

pub struct Application<'a> {
    terminal: tui::Terminal<CrosstermBackend<io::Stdout>>,
    active_folder_path: std::path::PathBuf,
    project_window: ProjectWindow<'a>,
    pub is_running: bool,
}

impl<'a> Application<'a> {
    pub fn new(path: std::path::PathBuf) -> Application<'a> {
        // load all project files from the path
        // TODO open the .pman folder instead and read projects from there
        let folder_result = std::fs::read_dir(path.as_path()).unwrap();
        let mut serialized_projects: Vec<Project> = vec![];
        for file in folder_result {
            let f = file.unwrap();
            if f.file_type().unwrap().is_file() {
                match f.path().extension() {
                    Some(ext) => {
                        if ext == utils::PROJECT_FILE_EXTENSION {
                            println!("Extension matches");
                            match match serde_json::from_str(
                                std::fs::read_to_string(f.path()).unwrap().as_str(),
                            ) {
                                Ok(result) => Some(result),
                                Err(_) => None,
                            } {
                                Some(project) => serialized_projects.push(project),
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                };
            }
        }
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut b_terminal = Terminal::new(backend).unwrap();
        b_terminal.clear().unwrap();
        Application {
            terminal: b_terminal,
            active_folder_path: path,
            project_window: ProjectWindow::new(serialized_projects),
            is_running: true,
        }
    }
    pub fn press_up(&mut self) {
        self.project_window.input_up();
    }
    pub fn press_down(&mut self) {
        self.project_window.input_down();
    }
    fn display_main_window(&mut self) {
        let text_active_path = Text::from(self.active_folder_path.to_str().unwrap());
        let project_window_ref = &mut self.project_window;
        self.terminal
            .draw(|f| {
                let window_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(5), Constraint::Percentage(95)].as_ref())
                    .split(f.size());

                let current_project_path = Paragraph::new(text_active_path);
                f.render_widget(current_project_path, window_layout[0]);
                project_window_ref.display(f);
            })
            .unwrap();
    }
    fn display_project_window(&mut self) {
        // self.terminal.draw(f: F)
    }
    pub fn update(&mut self) {
        self.display_main_window();
    }

    pub fn quit(&mut self) {
        self.is_running = false;
        self.terminal.clear();
        // TODO: Fix the weird look on the terminal after closing
    }
}

trait InformationDisplay {
    fn display_description(&self);
    fn display_full(&self);
}

trait Completable {
    fn complete(&self);
}

pub trait TaskContainer {
    fn add_task(&mut self, task_description: String);
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub description: String,
    pub active_tasks: Vec<Task>,
    pub completed_tasks: Vec<Task>,
}

impl Project {
    pub fn new(project_name: String) -> Project {
        Project {
            name: project_name,
            description: String::from("Sample description"),
            active_tasks: vec![],
            completed_tasks: vec![],
        }
    }
}

impl TaskContainer for Project {
    fn add_task(&mut self, task_desctiption: String) {
        let task = Task::new(task_desctiption);
        self.active_tasks.push(task);
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Task {
    pub description: String,
    pub time_spent: i32,
    pub estimate: i32,
    pub sub_tasks: Vec<Task>,
}

impl Task {
    fn new(task_description: String) -> Task {
        Task {
            description: task_description,
            time_spent: 0,
            estimate: 0,
            sub_tasks: vec![],
        }
    }
}

impl InformationDisplay for Task {
    fn display_description(&self) {}
    fn display_full(&self) {}
}

impl TaskContainer for Task {
    fn add_task(&mut self, task_desctiption: String) {
        let task = Task {
            description: task_desctiption,
            time_spent: 0,
            estimate: 0,
            sub_tasks: vec![],
        };
        self.sub_tasks.push(task);
    }
}
