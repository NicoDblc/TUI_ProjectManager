use serde::Deserialize;
use serde::Serialize;

use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;

use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, Widget};

pub struct Application {
    terminal: tui::Terminal<CrosstermBackend<io::Stdout>>,
    active_folder_path: std::path::PathBuf,
    current_folder_projects: Vec<Project>,
    pub is_running: bool,
}

// Application: Must display available projects from the ~/.project_manager or from the local folder

impl Application {
    pub fn new(path: std::path::PathBuf) -> Application {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let b_terminal = Terminal::new(backend).unwrap();
        let mut app = Application {
            terminal: b_terminal,
            active_folder_path: path,
            current_folder_projects: vec![],
            is_running: true,
        };
        app.terminal.clear();

        // TODO Initialize the path etc
        app
    }
    fn display_main_window(&mut self) {
        self.terminal.draw(|f| {
            let mut main_layout = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.size());
            let project_layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                .split(main_layout[0]);
            let block = Block::default().title("Projects").borders(Borders::ALL);
            f.render_widget(block, project_layout[0]);
            let block = Block::default()
                .title("Project description")
                .borders(Borders::ALL);
            f.render_widget(block, project_layout[1]);
            let task_layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(main_layout[1]);
            let block = Block::default().title("Active tasks").borders(Borders::ALL);
            f.render_widget(block, task_layout[0]);
            let block = Block::default()
                .title("Completed tasks")
                .borders(Borders::ALL);
            f.render_widget(block, task_layout[1]);
        });
    }
    fn display_project_window(&mut self) {
        // self.terminal.draw(f: F)
    }
    pub fn update(&mut self) {
        self.display_main_window();
    }
}

trait InformationDisplay {
    fn display_description(&self);
    fn display_full(&self);
}

trait Completable {
    fn complete(&self);
}

trait TaskContainer {
    fn add_task(&mut self, task_description: String);
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Project {
    name: String,
    description: String,
    tasks: Vec<Task>,
}

impl TaskContainer for Project {
    fn add_task(&mut self, task_desctiption: String) {
        let task = Task::new(task_desctiption);
        self.tasks.push(task);
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct Task {
    description: String,
    time_spent: i32,
    estimate: i32,
    sub_tasks: Vec<Task>,
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
