use serde::Deserialize;
use serde::Serialize;

use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;

use tui::layout::{Constraint, Direction, Layout};
use tui::text::Text;
use tui::widgets::Paragraph;

use crate::ui::{Drawable, InputMode, InputReceptor};
use crate::utils;

use crate::services::project_service::ProjectManagementService;
use crate::services::task_service::TaskService;
use crate::services::Service;
use crossterm::event::KeyCode;
use std::ops::Add;
use std::path::PathBuf;

enum SelectedWindow {
    Project,
    Task,
}

pub struct Application<'a> {
    terminal: tui::Terminal<CrosstermBackend<io::Stdout>>,
    active_folder_path: std::path::PathBuf,
    project_window: ProjectManagementService<'a>,
    task_window: TaskService,
    pub is_running: bool,
    selected_window: SelectedWindow,
}

impl<'a> Application<'a> {
    pub fn new(path: std::path::PathBuf) -> Application<'a> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut b_terminal = Terminal::new(backend).unwrap();
        b_terminal.clear().unwrap();
        let mut app_project_window = ProjectManagementService::new(path.clone());
        app_project_window.set_working_directory(
            path.clone()
                .join(String::from(".").add(utils::PROJECT_FILE_EXTENSION)),
        );
        Application {
            terminal: b_terminal,
            active_folder_path: path,
            project_window: app_project_window,
            task_window: TaskService::default(),
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
                let controls_string =
                    String::from(project_window_ref.get_controls_description().as_str());
                let controls_para = Paragraph::new(Text::from(controls_string));
                f.render_widget(controls_para, window_layout[2]);
                project_window_ref.display(f, window_layout[1]);
            })
            .unwrap();
    }

    fn display_tasks_window(&mut self) {
        let project_name = self
            .project_window
            .get_selected_project_path_name()
            .unwrap();
        let mut project_path = self.active_folder_path.clone(); //.with_file_name(project_name);
        project_path = project_path.join(String::from(".").add(utils::PROJECT_FILE_EXTENSION));
        project_path = project_path.with_file_name(project_name);
        project_path.set_extension(utils::PROJECT_FILE_EXTENSION);
        let text_active_path = Text::from(project_path.to_str().unwrap());
        let task_window_ref = &mut self.task_window;
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
                let controls_string =
                    String::from(task_window_ref.get_controls_description().as_str());
                let controls_para = Paragraph::new(Text::from(controls_string));
                f.render_widget(controls_para, window_layout[2]);
                task_window_ref.display(f, window_layout[1]);
            })
            .unwrap();
    }

    fn switch_to_window(&mut self, new_window: SelectedWindow) {
        self.selected_window = new_window;
        match self.selected_window {
            SelectedWindow::Project => {
                self.project_window =
                    ProjectManagementService::new(self.active_folder_path.clone());
            }
            SelectedWindow::Task => {
                match self.project_window.get_selected_project_path_name() {
                    Some(project_name) => {
                        self.task_window =
                            TaskService::new(self.active_folder_path.clone(), project_name)
                    }
                    None => self.selected_window = SelectedWindow::Project,
                };
            }
        }
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
                    InputMode::CommandMode => match key_code {
                        KeyCode::Char('q') => self.quit(),
                        KeyCode::Tab => self.switch_to_window(SelectedWindow::Task),
                        _ => {}
                    },
                    _ => {}
                }
            }
            SelectedWindow::Task => {
                self.task_window.handle_input_key(key_code);
                match self.task_window.get_input_mode() {
                    InputMode::CommandMode => match key_code {
                        KeyCode::Char('q') => self.quit(),
                        KeyCode::Tab => self.switch_to_window(SelectedWindow::Project),
                        _ => {}
                    },
                    _ => {}
                }
            }
        }
    }
    pub fn quit(&mut self) {
        self.is_running = false;
        match self.terminal.flush() {
            Err(e) => println!("Error when exiting program: {}", e),
            _ => {}
        };
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
    fn add_task(&mut self, task_name: String, task_description: String);
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

    pub fn write_project_full_path(&self, path_for_project: PathBuf) -> Result<(), std::io::Error> {
        let project_string = match serde_json::to_string(self) {
            Ok(p_string) => p_string,
            Err(e) => {
                return Result::Err(std::io::Error::from(e));
            }
        };
        match std::fs::write(path_for_project, project_string) {
            Ok(()) => Ok(()),
            Err(e) => Result::Err(e),
        }
    }
}

impl TaskContainer for Project {
    fn add_task(&mut self, task_name: String, task_description: String) {
        let task = Task::new(task_name, task_description);
        self.active_tasks.push(task);
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Task {
    pub name: String,
    pub description: String,
    pub time_spent: i32,
    pub estimate: i32,
    pub sub_tasks: Vec<Task>,
}

impl Task {
    pub fn new(task_name: String, task_description: String) -> Task {
        Task {
            name: task_name,
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
    fn add_task(&mut self, task_name: String, task_description: String) {
        let task = Task {
            name: task_name,
            description: task_description,
            time_spent: 0,
            estimate: 0,
            sub_tasks: vec![],
        };
        self.sub_tasks.push(task);
    }
}
