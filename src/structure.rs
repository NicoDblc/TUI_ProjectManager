use serde::Deserialize;
use serde::Serialize;

use std::io;
use tui::backend::CrosstermBackend;
use tui::{Frame, Terminal};

use tui::layout::{Constraint, Direction, Layout};
use tui::text::Text;
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use crate::ui::{InputMode, ProjectWindow, Window};
use crate::utils;

use crossterm::event::Event::Key;
use crossterm::event::KeyCode;
use std::io::Stdout;
use std::ops::Add;
use std::ptr::eq;

enum SelectedWindow {
    Project,
    Task,
}

pub struct Application<'a> {
    terminal: tui::Terminal<CrosstermBackend<io::Stdout>>,
    active_folder_path: std::path::PathBuf,
    project_window: ProjectWindow<'a>,
    pub is_running: bool,
    selected_window: SelectedWindow,
}

impl<'a> Application<'a> {
    pub fn new(path: std::path::PathBuf) -> Application<'a> {
        let serialized_projects = utils::get_projects_in_path(path.clone());
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut b_terminal = Terminal::new(backend).unwrap();
        b_terminal.clear().unwrap();
        let mut app_project_window = ProjectWindow::new(serialized_projects);
        app_project_window.set_program_work_path(path.clone());
        Application {
            terminal: b_terminal,
            active_folder_path: path,
            project_window: app_project_window,
            is_running: true,
            selected_window: SelectedWindow::Project,
        }
    }
    fn display_main_window(&mut self) {
        let text_active_path = Text::from(self.active_folder_path.to_str().unwrap());
        let project_window_ref = &mut self.project_window;
        self.terminal
            .draw(|f| {
                let window_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Percentage(5),
                            Constraint::Percentage(90),
                            Constraint::Percentage(5),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());
                let current_project_path = Paragraph::new(text_active_path);
                f.render_widget(current_project_path, window_layout[0]);
                let controls_string = String::from("q - quit     ")
                    .add(project_window_ref.get_controls_description().as_str());
                let controls_para = Paragraph::new(Text::from(controls_string));
                f.render_widget(controls_para, window_layout[2]);
                project_window_ref.display(f, window_layout[1]);
            })
            .unwrap();
    }
    fn display_tasks_window(&mut self) {
        // TODO: Implement
    }
    pub fn update(&mut self) {
        match self.selected_window {
            SelectedWindow::Project => {
                self.display_main_window();
            }
            SelectedWindow::Task => {
                self.display_tasks_window();
            }
        }
    }
    pub fn handle_inputs(&mut self, key_code: KeyCode) {
        match self.selected_window {
            SelectedWindow::Project => {
                self.project_window.handle_input_key(key_code);
                match self.project_window.get_input_mode() {
                    InputMode::CommandMode => {
                        if key_code == KeyCode::Char('q') {
                            self.quit();
                        }
                    }
                    _ => {}
                }
            }
            SelectedWindow::Task => {
                // TODO: Handle/create task window
            }
        }
    }
    pub fn quit(&mut self) {
        self.is_running = false;
        self.terminal.flush();
    }
}

trait InformationDisplay {
    fn get_description(&self) -> String;
    fn get_name(&self) -> String;
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
    fn add_task(&mut self, task_description: String) {
        let task = Task::new(task_description);
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
    pub fn new(task_description: String) -> Task {
        Task {
            description: task_description,
            time_spent: 0,
            estimate: 0,
            sub_tasks: vec![],
        }
    }
}

impl InformationDisplay for Task {
    fn get_description(&self) -> String {
        self.description.clone()
    }

    fn get_name(&self) -> String {
        self.description.clone()
    }
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
