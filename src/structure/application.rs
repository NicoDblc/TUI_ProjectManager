use crate::services::project_s::ProjectManagementService;
use crate::services::task_s::TaskService;
use crate::ui::{Drawable, InputMode, InputReceptor};
use crate::utils;
use crossterm::event::KeyCode;
use std::io;
use std::ops::Add;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::text::Text;
use tui::widgets::Paragraph;
use tui::Terminal;

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
        let app_project_window = ProjectManagementService::new(path.clone());
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
        let mut project_path = self.active_folder_path.clone();
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

    pub fn update(&mut self) {
        self.display_main_window();
    }
    pub fn handle_inputs(&mut self, key_code: KeyCode) {
        self.project_window.handle_input_key(key_code);
        match self.project_window.get_input_mode() {
            InputMode::CommandMode => match key_code {
                KeyCode::Char('q') => self.quit(),
                _ => {}
            },
            InputMode::SubWindowInputs => match key_code {
                KeyCode::Char('q') => self.quit(),
                _ => {}
            },
            _ => {}
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
