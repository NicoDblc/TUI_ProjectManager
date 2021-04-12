use crate::services::project_s::ProjectManagementService;
use crate::services::task_s::TaskService;
use crate::ui::{Drawable, InputMode, InputReceptor};
use crate::utils;
use crossterm::event::{read, KeyCode, KeyEvent, KeyModifiers};
use std::io;
use std::ops::Add;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::text::Text;
use tui::widgets::Paragraph;
use tui::Terminal;
use crate::services::Service;
use crate::structure::project::Project;
use crate::structure::task::Task;
use crossterm::event::Event::Key;

enum SelectedWindow {
    Project,
    Task,
}

#[derive(Default)]
pub struct ApplicationState {
    pub(crate) folder_path: std::path::PathBuf,
    pub(crate) selected_project: Project,
    pub(crate) selected_task: Task,
}

pub struct Application<'a> {
    terminal: tui::Terminal<CrosstermBackend<io::Stdout>>,
    active_folder_path: std::path::PathBuf,
    project_window: ProjectManagementService<'a>,
    task_window: TaskService,
    pub is_running: bool,
    selected_window: SelectedWindow,
    services: Vec<Box<dyn Service>>,
    service_index: u32,
    application_state: ApplicationState
}

// todo: give to ability to the service to update it's status based on the application state.
// It gives the opportunity to the task manager to get the current project.
// It gives the current project the opportunity to get working path.
// todo: provide a standard access way and a service update access.

impl<'a> Application<'a> {
    pub fn new(path: std::path::PathBuf) -> Application<'a> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut b_terminal = Terminal::new(backend).unwrap();
        b_terminal.clear().unwrap();
        let app_project_window = ProjectManagementService::new(path.clone());
        Application {
            terminal: b_terminal,
            active_folder_path: path.clone(),
            project_window: app_project_window,
            task_window: TaskService::default(),
            is_running: true,
            selected_window: SelectedWindow::Project,
            services: vec![Box::new(ProjectManagementService::new(path.clone())), Box::new(TaskService::default())],
            service_index: 0,
            application_state: ApplicationState{
                folder_path: path.clone(),
                selected_project: Project::default(),
                selected_task: Task::default(),
            }
        }
    }

    fn display_current(&mut self){
        let text_active_path = Text::from(self.active_folder_path.to_str().unwrap());
        let mut service_ref;
        let service_index = self.service_index as usize;
        if self.services.len() > 0 {
            service_ref = Box::new(self.services[service_index].as_ref());
            self.terminal.draw(|f| {
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
                service_ref.display(f, window_layout[1]);
                let controls_string =
                    String::from(service_ref.get_input_possibilities().as_str());
                let controls_para = Paragraph::new(Text::from(controls_string));
                f.render_widget(controls_para, window_layout[2]);
            }).unwrap();
        }
    }

    fn increment_service_index(&mut self) {
        self.service_index = (self.service_index + 1) % (self.services.len()as u32)   ;
    }

    fn decrement_service_index(&mut self) {
        self.service_index = (self.service_index - 1) % (self.services.len() as u32);
    }

    pub fn update(&mut self) {
        self.display_current();
    }

    pub fn handle_inputs(&mut self, key_event: KeyEvent) {
        if !self.services[self.service_index as usize].handle_input(key_event.code) {
            match key_event.code {
                KeyCode::Char('q') => self.quit(),
                KeyCode::Tab =>  {
                    self.services[self.service_index as usize].update_application_state(&mut self.application_state);
                    match key_event.modifiers {
                        KeyModifiers::SHIFT => {
                            self.decrement_service_index();
                        }
                        KeyModifiers::NONE => {
                            self.increment_service_index();
                        }
                        (_)=>{}
                    }
                    self.services[self.service_index as usize].update_application_state(&mut self.application_state);
                }
                _ => {}
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
