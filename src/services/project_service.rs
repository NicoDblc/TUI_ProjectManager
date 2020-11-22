use crate::structure::Project;
use crate::ui::{
    Completable, DisplayList, Drawable, InputMode, InputReceptor, InputReturn, PopupBinaryChoice,
    PopupInputWindow, PopupMessageWindow,
};
use crate::utils::get_projects_in_path;
use crate::{services, utils};
use crossterm::event::KeyCode;
use std::io::{Error, Stdout};
use std::ops::Add;
use std::path::PathBuf;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::text::Text;
use tui::widgets::{Block, Borders, List, ListItem, Paragraph};
use tui::Frame;

enum ProjectInputType {
    ProjectAdd,
    ProjectDescriptionEdit,
    ProjectNameEdit,
}

pub struct ProjectManagementService<'a> {
    // Everything that is contained in the draw call for the main window
    projects_to_display: DisplayList<Project>,
    selected_project_active_tasks: Vec<ListItem<'a>>,
    selected_project_completed_tasks: Vec<ListItem<'a>>,
    project_input_popup: PopupInputWindow,
    input_mode: InputMode,
    input_type: ProjectInputType,
    program_work_path: PathBuf,
    message_popup: PopupMessageWindow,
    delete_project_popup: PopupBinaryChoice,
}

impl<'a> ProjectManagementService<'a> {
    pub fn new(working_path: PathBuf) -> ProjectManagementService<'a> {
        let mut project_window = ProjectManagementService {
            projects_to_display: DisplayList::from(utils::get_projects_in_path(
                working_path.clone(),
            )),
            selected_project_active_tasks: Vec::new(),
            selected_project_completed_tasks: Vec::new(),
            project_input_popup: PopupInputWindow::default(),
            input_mode: InputMode::CommandMode,
            input_type: ProjectInputType::ProjectAdd,
            program_work_path: working_path,
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

    fn reload_projects(&mut self) {
        self.update_projects(utils::get_projects_in_path(self.program_work_path.clone()));
    }

    fn update_project_selection(&mut self) {
        self.selected_project_active_tasks = self.projects_to_display.array
            [self.projects_to_display.state.selected().unwrap()]
        .active_tasks
        .clone()
        .into_iter()
        .map(|a| ListItem::new(Text::from(a.name)))
        .collect();
        self.selected_project_completed_tasks = self.projects_to_display.array
            [self.projects_to_display.state.selected().unwrap()]
        .completed_tasks
        .clone()
        .into_iter()
        .map(|a| ListItem::new(Text::from(a.name)))
        .collect();
    }

    fn create_popup_with_message(&mut self, message: String) {
        self.message_popup = PopupMessageWindow::new(message);
    }

    fn add_project_request(&mut self) {
        self.input_mode = InputMode::WriteMode;
        self.input_type = ProjectInputType::ProjectAdd;
        self.project_input_popup = PopupInputWindow::new(String::from("Insert project name"));
    }

    fn write_project_to_disk(&self, project_to_write: Project) -> Result<(), Error> {
        let mut project_file_path = self.program_work_path.join(project_to_write.name.clone());
        project_file_path.set_extension(utils::PROJECT_FILE_EXTENSION);
        project_to_write.write_project_full_path(project_file_path)
    }

    fn delete_selected_project(&mut self) {
        if self.projects_to_display.array.len() > 0 {
            let popup_description =
                String::from("Delete project: ").add(self.get_selected_project_name().as_str());
            self.delete_project_popup = PopupBinaryChoice::new(popup_description);
            self.input_mode = InputMode::WriteMode;
        }
    }

    fn edit_selected_project_name(&mut self) {
        if self.projects_to_display.array.len() > 0 {
            self.input_mode = InputMode::WriteMode;
            self.input_type = ProjectInputType::ProjectNameEdit;
            self.project_input_popup = PopupInputWindow::new(String::from("Edit project name"));
            self.project_input_popup.set_input_string(
                self.projects_to_display.array[self.projects_to_display.state.selected().unwrap()]
                    .name
                    .clone(),
            );
        }
    }

    fn edit_selected_project_description(&mut self) {
        if self.projects_to_display.array.len() > 0 {
            self.input_mode = InputMode::WriteMode;
            self.input_type = ProjectInputType::ProjectDescriptionEdit;
            self.project_input_popup =
                PopupInputWindow::new(String::from("Edit project description"));
            self.project_input_popup.set_input_string(
                self.projects_to_display.array[self.projects_to_display.state.selected().unwrap()]
                    .description
                    .clone(),
            );
        }
    }

    // Unsafe. TODO: Fix
    fn get_selected_project_name(&self) -> String {
        self.projects_to_display.array[self.projects_to_display.state.selected().unwrap()]
            .clone()
            .name
    }

    pub fn get_selected_project_path_name(&self) -> Option<String> {
        match self.projects_to_display.state.selected() {
            Some(val) => Option::Some(self.projects_to_display.array[val].clone().name),
            None => None,
        }
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
                            match utils::delete_project_of_name(
                                self.get_selected_project_name(),
                                self.program_work_path.clone(),
                            ) {
                                Ok(()) => {}
                                Err(e) => {
                                    self.create_popup_with_message(e.to_string());
                                }
                            };
                            self.update_projects(utils::get_projects_in_path(
                                self.program_work_path.clone(),
                            ));
                        }
                        self.delete_project_popup.set_active(false);
                    }
                } else {
                    self.input_mode = InputMode::CommandMode;
                    self.handle_input_key(key_code);
                }
            }
        }

        if self.project_input_popup.is_active() & self.project_input_popup.is_completed() {
            match self.input_type {
                ProjectInputType::ProjectAdd => {
                    let new_project = Project::new(self.project_input_popup.get_input_data());
                    match self.write_project_to_disk(new_project) {
                        Ok(_) => {
                            self.project_input_popup.set_active(false);
                            self.update_projects(get_projects_in_path(
                                self.program_work_path.clone(),
                            ));
                            self.input_mode = InputMode::CommandMode;
                        }
                        Err(e) => {
                            self.create_popup_with_message(
                                e.to_string()
                                    .add(" With path: ")
                                    .add(self.program_work_path.clone().to_str().unwrap()),
                            );
                            self.project_input_popup.reset_completion();
                            return;
                        }
                    };
                }
                ProjectInputType::ProjectNameEdit => {
                    let mut project = self.projects_to_display.array
                        [self.projects_to_display.state.selected().unwrap()]
                    .clone();
                    let mut original_path = self.program_work_path.clone().join(project.name);
                    original_path.set_extension(utils::PROJECT_FILE_EXTENSION);
                    let mut new_path = self
                        .program_work_path
                        .clone()
                        .join(self.project_input_popup.get_input_data());
                    new_path.set_extension(utils::PROJECT_FILE_EXTENSION);
                    project.name = self.project_input_popup.get_input_data();
                    match std::fs::rename(original_path, new_path) {
                        Ok(()) => {
                            match self.write_project_to_disk(project) {
                                Ok(()) => {}
                                Err(e) => {
                                    self.create_popup_with_message(e.to_string());
                                    self.project_input_popup.reset_completion();
                                    return;
                                }
                            };
                            self.reload_projects();
                            self.project_input_popup.set_active(false);
                        }
                        Err(e) => {
                            self.create_popup_with_message(e.to_string());
                            self.project_input_popup.reset_completion();
                            return;
                        }
                    }
                }
                ProjectInputType::ProjectDescriptionEdit => {
                    let mut project = self.projects_to_display.array
                        [self.projects_to_display.state.selected().unwrap()]
                    .clone();
                    let new_description = self.project_input_popup.get_input_data();
                    project.description = new_description;
                    match self.write_project_to_disk(project) {
                        Ok(_) => {
                            self.reload_projects();
                            self.project_input_popup.set_active(false);
                        }
                        Err(e) => {
                            self.create_popup_with_message(e.to_string());
                            self.project_input_popup.reset_completion();
                            return;
                        }
                    };
                }
            };
        }
    }

    fn get_controls_description(&self) -> String {
        if self.message_popup.is_active() {
            return self.message_popup.get_controls_description();
        } else if self.delete_project_popup.is_active() {
            return self.delete_project_popup.get_controls_description();
        } else if self.project_input_popup.is_active() {
            return self.project_input_popup.get_controls_description();
        }
        String::from("Q: Quit | A: Add project | D: Delete project | E: Edit Project Description | N: Edit Project name | Tab: Go to Tasks")
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

impl<'a> services::Service for ProjectManagementService<'a> {
    fn set_working_directory(&mut self, path: PathBuf) {
        self.program_work_path = path;
    }
}
