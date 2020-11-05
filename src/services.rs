use std::path::{Path, PathBuf};
use crate::{ui, utils};
use crate::structure;
use crate::structure::Project;
use tui::widgets::{ListItem, Block, List, Borders, Paragraph};
use crate::ui::{InputMode, InputReceptor, Drawable, DisplayList, PopupInputWindow, InputReturn, PopupMessageWindow, Completable, PopupBinaryChoice};
use tui::backend::CrosstermBackend;
use tui::Frame;
use tui::text::Text;
use std::io::{Stdout, Error};
use tui::layout::{Rect, Layout, Direction, Constraint};
use crossterm::event::KeyCode;
use crate::utils::{get_projects_in_path, delete_project_of_name};
use std::ops::Add;
use crate::Event::Input;


pub trait Service {
    fn set_working_directory(&mut self, path: PathBuf);

}


pub struct ProjectManagementService<'a> {
    // Everything that is contained in the draw call for the main window
    projects_to_display: DisplayList<Project>,
    selected_project_active_tasks: Vec<ListItem<'a>>,
    selected_project_completed_tasks: Vec<ListItem<'a>>,
    project_input_popup: PopupInputWindow,
    input_mode: InputMode,
    program_work_path: PathBuf,
    message_popup: PopupMessageWindow,
    delete_project_popup: PopupBinaryChoice,
}

impl<'a> ProjectManagementService<'a> {
    pub fn new(projects: Vec<Project>) -> ProjectManagementService<'a> {
        let mut project_window = ProjectManagementService {
            projects_to_display: DisplayList::from(projects),
            selected_project_active_tasks: Vec::new(),
            selected_project_completed_tasks: Vec::new(),
            project_input_popup: PopupInputWindow::default(),
            input_mode: InputMode::CommandMode,
            program_work_path: PathBuf::new(),
            message_popup: PopupMessageWindow::default(),
            delete_project_popup: PopupBinaryChoice::default(),
        };
        if project_window.projects_to_display.array.len() > 0 {
            project_window.update_project_selection();
        }

        project_window
    }

    fn update_projects(&mut self, projects: Vec<Project>) {
        self.projects_to_display = DisplayList::from(projects);
    }

    fn next_project_selection(&mut self) {}

    fn update_project_selection(&mut self) {
        self.selected_project_active_tasks = self.projects_to_display.array
            [self.projects_to_display.state.selected().unwrap()]
            .active_tasks
            .clone()
            .into_iter()
            .map(|a| ListItem::new(Text::from(a.description)))
            .collect();
        self.selected_project_completed_tasks = self.projects_to_display.array
            [self.projects_to_display.state.selected().unwrap()]
            .completed_tasks
            .clone()
            .into_iter()
            .map(|a| ListItem::new(Text::from(a.description)))
            .collect();
    }

    fn add_project_request(&mut self) {
        self.input_mode = InputMode::WriteMode;
        self.project_input_popup = PopupInputWindow::new(String::from(
            "Insert project name",
        ));
    }

    fn delete_selected_project(&mut self) {
        if self.projects_to_display.array.len() > 0 {
            let popup_description = String::from("Delete project: ")
                .add(self.get_selected_project_name().as_str());
            self.delete_project_popup = PopupBinaryChoice::new(popup_description);
            self.input_mode = InputMode::WriteMode;
        }
    }

    fn edit_selected_project_name(&mut self) {
        // TODO: Implement (with pop up)
        // New popup same type as input
    }

    fn edit_selected_project_description(&mut self) {
        // TODO: Implement (with pop up)
        // New popup same type asm input
    }

    fn get_selected_project_name(&self) -> String {
        self.projects_to_display
            .array[self.projects_to_display.state.selected().unwrap()].clone().name
    }
}

impl<'a> InputReceptor for ProjectManagementService<'a> {
    fn handle_input_key(&mut self, key_code: KeyCode) {
        match self.input_mode {
            InputMode::CommandMode => match key_code {
                KeyCode::Up => {
                    self.projects_to_display.previous();
                    self.update_project_selection();
                }
                KeyCode::Down => {
                    self.projects_to_display.next();
                    self.update_project_selection();
                }
                KeyCode::Char('a') => {
                    self.add_project_request();
                }
                KeyCode::Char('d') => {
                    self.delete_selected_project();
                }
                KeyCode::Char('e') => {
                    self.edit_selected_project_description();
                }
                KeyCode::Char('n') => {
                    self.edit_selected_project_name();
                }
                _ => {}
            },
            InputMode::WriteMode => {
                if self.message_popup.is_active() {
                    self.message_popup.handle_input_key(key_code);
                    if self.message_popup.is_completed() {
                        self.message_popup.set_active(false);
                    }
                } else if self.project_input_popup.is_active() {
                    self.project_input_popup.handle_input_key(key_code);
                } else if self.delete_project_popup.is_active() {
                    self.delete_project_popup.handle_input_key(key_code);
                    if self.delete_project_popup.is_completed() {
                        if self.delete_project_popup.get_choice() {
                            utils::delete_project_of_name(self.get_selected_project_name(), self.program_work_path.clone());
                            self.update_projects(utils::get_projects_in_path(self.program_work_path.clone()));
                        }
                        self.delete_project_popup.set_active(false);
                    }
                }
                else {
                    self.input_mode = InputMode::CommandMode;
                    self.handle_input_key(key_code);
                }
            }
        }

        if self.project_input_popup.is_active() & self.project_input_popup.is_completed() {
            let new_project = Project::new(self.project_input_popup.get_input_data());
            let project_string = match serde_json::to_string(&new_project) {
                Ok(p_string) => p_string,
                Err(e) => {
                    self.message_popup = PopupMessageWindow::new(String::from("Error: ").add(e.to_string().as_str()));
                    return;
                }
            };
            let mut project_file_path = self.program_work_path.join(new_project.name);
            project_file_path.set_extension(utils::PROJECT_FILE_EXTENSION);
            match std::fs::write(project_file_path, project_string){
                Ok(_) => {
                    self.project_input_popup.set_active(false);
                    self.update_projects(get_projects_in_path(self.program_work_path.clone()));
                    self.input_mode = InputMode::CommandMode;
                },
                Err(e) => {
                    self.message_popup = PopupMessageWindow::new(String::from("Error: ").add(e.to_string().as_str()));
                    self.project_input_popup.reset_completion();
                    return;
                }
            };
        }
    }

    fn get_controls_description(&self) -> String {
        String::from("a - Add project     d - Delete project     e - Edit Project Description     n - Edit Project name")
    }

    fn get_input_mode(&self) -> InputMode {
        match self.input_mode {
            InputMode::CommandMode => InputMode::CommandMode,
            InputMode::WriteMode => InputMode::WriteMode,
        }
    }
}

impl<'a> Drawable for ProjectManagementService<'a> {
    fn display(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, layout: Rect) {
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(layout);

        let project_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
            .split(main_layout[0]);
        let block = Block::default().title("Projects").borders(Borders::ALL);
        let p_list = List::new::<Vec<ListItem>>(
            self.projects_to_display
                .array
                .clone()
                .into_iter()
                .map(|p| ListItem::new(Text::from(p.name)))
                .collect(),
        )
            .block(block)
            .highlight_style(
                tui::style::Style::default()
                    .bg(tui::style::Color::Green)
                    .add_modifier(tui::style::Modifier::BOLD),
            )
            .highlight_symbol("-> ");
        frame.render_stateful_widget(
            p_list,
            project_layout[0],
            &mut self.projects_to_display.state.clone(),
        );

        let block = Block::default()
            .title("Project description")
            .borders(Borders::ALL);
        let p_description = match self.projects_to_display.array.len() > 0 {
            true => Paragraph::new(
                self.projects_to_display.array[self.projects_to_display.state.selected().unwrap()]
                    .clone()
                    .description,
            )
                .block(block),
            false => Paragraph::new("").block(block),
        };
        frame.render_widget(p_description, project_layout[1]);

        let task_layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(main_layout[1]);
        let block = Block::default().title("Active tasks").borders(Borders::ALL);
        let current_task_list = List::new(self.selected_project_active_tasks.clone()).block(block);
        frame.render_widget(current_task_list, task_layout[0]);
        let block = Block::default()
            .title("Completed tasks")
            .borders(Borders::ALL);
        let completed_tasks_list =
            List::new(self.selected_project_completed_tasks.clone()).block(block);
        frame.render_widget(completed_tasks_list, task_layout[1]);
        if self.project_input_popup.is_active() {
            self.project_input_popup.display(frame, layout);
        }
        if self.delete_project_popup.is_active() {
            self.delete_project_popup.display(frame, layout);
        }
        if self.message_popup.is_active() {
            self.message_popup.display(frame, layout);
        }
    }
}

impl<'a> Service for ProjectManagementService<'a> {
    fn set_working_directory(&mut self, path: PathBuf) {
        self.program_work_path = path;
    }
}