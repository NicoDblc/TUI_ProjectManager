use serde::Deserialize;
use serde::Serialize;

use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;

use tui::layout::{Constraint, Direction, Layout};
use tui::text::Text;
use tui::widgets::{Block, Borders, List, ListItem, Paragraph, Widget};

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
        app.terminal.clear().unwrap();
        for i in 1..10 {
            app.current_folder_projects
                .push(Project::new(i.to_string()));
        }

        // TODO Initialize the path etc
        app
    }
    fn display_main_window(&mut self) {
        let text_active_path = Text::from(self.active_folder_path.to_str().unwrap());
        let mut projects_list_viz: Vec<ListItem> = vec![];
        for p in &self.current_folder_projects {
            projects_list_viz.push(ListItem::new(Text::from(p.name.clone())));
        }
        // get current index of the selected Project and Create a Text item containg the description
        // This shit is getting ugly, I don't like it
        self.terminal
            .draw(|f| {
                let window_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(5), Constraint::Percentage(95)].as_ref())
                    .split(f.size());

                let current_project_path = Paragraph::new(text_active_path);
                f.render_widget(current_project_path, window_layout[0]);
                let mut main_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(1)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(window_layout[1]);

                let project_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                    .split(main_layout[0]);
                let block = Block::default().title("Projects").borders(Borders::ALL);
                //f.render_widget(block, project_layout[0]);
                // Project lists
                let p_list = List::new(projects_list_viz).block(block);
                f.render_widget(p_list, project_layout[0]);

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
            })
            .unwrap();
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

impl Project {
    fn new(project_name: String) -> Project {
        Project {
            name: project_name,
            description: String::from("Sample description"),
            tasks: vec![],
        }
    }
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
