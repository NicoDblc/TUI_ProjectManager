use serde::Deserialize;
use serde::Serialize;

use std::io;
use tui::backend::CrosstermBackend;
use tui::Terminal;

use std::ffi::OsStr;
use std::fs::ReadDir;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::Text;
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Widget};

static PROJECT_FILE_EXTENSION: &str = "pman";
// Application: Must display available projects from the ~/.project_manager or from the local folder
#[derive(Default)]
struct DisplayList<T> {
    state: ListState,
    array: Vec<T>,
}

impl<T> DisplayList<T> {
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i < self.array.len() {
                    i + 1
                } else {
                    0
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i > 0 {
                    i - 1
                }else {
                    0
                }

            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}
pub struct Application {
    terminal: tui::Terminal<CrosstermBackend<io::Stdout>>,
    active_folder_path: std::path::PathBuf,
    project_list: DisplayList<Project>,
    pub is_running: bool,
}

impl Application {
    pub fn new(path: std::path::PathBuf) -> Application {
        // load all project files from the path
        // TODO open the .pman folder instead and read projects from there
        let folder_result = std::fs::read_dir(path.as_path()).unwrap();
        let mut serialized_projects: Vec<Project> = vec![];
        for file in folder_result {
            let f = file.unwrap();
            if f.file_type().unwrap().is_file() {
                let extension = match f.path().extension() {
                    Some(ext) => {
                        if ext == PROJECT_FILE_EXTENSION {
                            match match serde_json::from_str(
                                std::fs::read_to_string(f.path()).unwrap().as_str(),
                            ) {
                                Ok(result) => Some(result),
                                Err(E) => None,
                            } {
                                Some(project) => serialized_projects.push(project),
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                };
                // if f.path().extension().unwrap() == PROJECT_FILE_EXTENSION {
                //     match match serde_json::from_str(
                //         std::fs::read_to_string(f.path()).unwrap().as_str(),
                //     ) {
                //         Ok(result) => Some(result),
                //         Err(E) => None,
                //     } {
                //         Some(project) => serialized_projects.push(project),
                //         _ => {}
                //     };
                // }
            }
        }

        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut b_terminal = Terminal::new(backend).unwrap();
        b_terminal.clear().unwrap();
        let app = Application {
            terminal: b_terminal,
            active_folder_path: path,
            project_list: DisplayList::default(),
            is_running: true,
        };
        // adding test projects
        for i in 1..10 {
            // app.current_folder_projects
            //     .push(Project::new(i.to_string()));
        }

        app
    }
    pub fn press_up(&mut self) {
        self.project_list.previous();
    }
    pub fn press_down(&mut self) {
        self.project_list.next();
    }
    fn display_main_window(&mut self) {
        let text_active_path = Text::from(self.active_folder_path.to_str().unwrap());
        let mut projects_list_viz: Vec<ListItem> = vec![];
        for p in &self.project_list.array {
            projects_list_viz.push(ListItem::new(Text::from(p.name.clone())));
        }

        let project_list = &self.project_list;
        // This shit is getting ugly, I don't like it
        self.terminal
            .draw(|f| {
                let window_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(5), Constraint::Percentage(95)].as_ref())
                    .split(f.size());

                let current_project_path = Paragraph::new(text_active_path);
                f.render_widget(current_project_path, window_layout[0]);
                let main_layout = Layout::default()
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
                let p_list = List::new(projects_list_viz)
                    .block(block)
                    .highlight_style(
                        tui::style::Style::default()
                            .bg(tui::style::Color::DarkGray)
                            .add_modifier(tui::style::Modifier::BOLD),
                    )
                    .highlight_symbol("-> ");
                f.render_widget(p_list, project_layout[0]);

                let block = Block::default()
                    .title("Project description")
                    .borders(Borders::ALL);
                let p_description = match project_list.array.len() > 0 {
                    true => Paragraph::new(
                        project_list.array[project_list.state.selected().unwrap()]
                            .clone()
                            .description,
                    )
                    .block(block),
                    false => Paragraph::new("").block(block),
                };

                f.render_widget(p_description, project_layout[1]);

                let task_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(0)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(main_layout[1]);
                let block = Block::default().title("Active tasks").borders(Borders::ALL);
                // TODO create List with task items

                let new_list: Vec<ListItem> = match project_list.array.len() > 0 {
                    true => {
                        project_list.array
                            [project_list.state.selected().unwrap()]
                            .active_tasks
                            .clone()
                            .into_iter()
                            .map(|a| ListItem::new(Text::from(a.description)))
                            .collect()
                    },
                    false => {
                        Vec::new()
                    }
                };

                let current_task_list = List::new(new_list).block(block);
                f.render_widget(current_task_list, task_layout[0]);

                let block = Block::default()
                    .title("Completed tasks")
                    .borders(Borders::ALL);
                let new_list: Vec<ListItem> = match project_list.array.len() > 0 {
                    true => {
                        project_list.array
                            [project_list.state.selected().unwrap()]
                            .completed_tasks
                            .clone()
                            .into_iter()
                            .map(|a| ListItem::new(Text::from(a.description)))
                            .collect()
                    }
                    false => {
                        Vec::new()
                    }
                };

                let completed_tasks_list = List::new(new_list).block(block);
                f.render_widget(completed_tasks_list, task_layout[1]);
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
    active_tasks: Vec<Task>,
    completed_tasks: Vec<Task>,
}

impl Project {
    fn new(project_name: String) -> Project {
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

struct A {
    word: String,
    number: u32,
}

fn some_function() {
    let some_A: Vec<A> = Vec::new();
    let phrases: Vec<String> = Vec::new();
}
