use crate::popups::{InputWindow, MessageWindow};
use crate::services::task_s::TaskInputChoice::{AddName, EditDescription};
use crate::services::Service;
use crate::structure::project::*;
use crate::structure::task::Task;
use crate::structure::*;
use crate::ui::InputMode::CommandMode;
use crate::ui::{Completable, DisplayList, Drawable, InputMode, InputReceptor, InputReturn};
use crate::utils;
use crossterm::event::KeyCode;
use std::io::Stdout;
use std::ops::Add;
use std::path::PathBuf;
use tui::backend::CrosstermBackend;
use tui::layout::Direction::{Horizontal, Vertical};
use tui::layout::{Constraint, Layout, Rect};
use tui::text::Text;
use tui::widgets::{Block, Borders, List, ListItem, Paragraph, Wrap};
use tui::Frame;

#[derive(SmartDefault)]
enum TaskInputChoice {
    #[default]
    AddName,
    EditDescription,
}

#[derive(Default)]
pub struct TaskService {
    working_path: PathBuf,
    selected_project: Project,
    active_tasks_list: DisplayList<Task>,
    completed_tasks_list: DisplayList<Task>,
    focused_on_active: bool,
    input_mode: InputMode,
    input_popup: InputWindow,
    input_popup_type: TaskInputChoice,
    message_popup: MessageWindow,
}

impl TaskService {
    pub fn new(working_path: PathBuf, project_name: String) -> TaskService {
        let mut project_path = working_path
            .join(String::from('.').add(utils::PROJECT_FILE_EXTENSION))
            .with_file_name(project_name);
        project_path.set_extension(utils::PROJECT_FILE_EXTENSION);
        let loaded_project = match load_project_from_path(project_path.clone()) {
            Ok(loaded_project) => loaded_project,
            Err(_) => Project::default(),
        };
        TaskService {
            working_path: project_path,
            selected_project: loaded_project.clone(),
            active_tasks_list: DisplayList::from(loaded_project.active_tasks.clone()),
            completed_tasks_list: DisplayList::from(loaded_project.completed_tasks.clone()),
            focused_on_active: true,
            input_mode: InputMode::CommandMode,
            input_popup: InputWindow::default(),
            input_popup_type: TaskInputChoice::AddName,
            message_popup: MessageWindow::default(),
        }
    }

    fn add_task_command(&mut self) {
        self.input_popup_type = AddName;
        self.input_mode = InputMode::WriteMode;
        self.input_popup = InputWindow::new(String::from("Enter Task Name"));
    }

    fn edit_task_description(&mut self) {
        let input_string = match self.focused_on_active {
            true => match self.active_tasks_list.state.selected() {
                Some(val) => self.active_tasks_list.array[val].description.clone(),
                None => String::from(""),
            },
            false => match self.completed_tasks_list.state.selected() {
                Some(val) => self.completed_tasks_list.array[val].description.clone(),
                None => String::from(""),
            },
        };
        self.input_popup_type = EditDescription;
        self.input_mode = InputMode::WriteMode;
        self.input_popup = InputWindow::new(String::from("Edit tasks description"));
        self.input_popup.set_input_string(input_string);
    }

    fn mark_selected_task_as_completed(&mut self) {
        match self.active_tasks_list.state.selected() {
            Some(val) => {
                self.completed_tasks_list
                    .array
                    .push(self.active_tasks_list.array[val].clone());
                self.active_tasks_list.array.remove(val);
                self.selected_project.active_tasks = self.active_tasks_list.array.clone();
                self.selected_project.completed_tasks = self.completed_tasks_list.array.clone();
                match self
                    .selected_project
                    .write_project_full_path(self.working_path.clone())
                {
                    Ok(_) => {}
                    Err(e) => self.create_message_popup(e.to_string()),
                };
            }
            None => {}
        }
    }

    fn mark_selected_task_as_uncompleted(&mut self) {
        match self.completed_tasks_list.state.selected() {
            Some(val) => {
                self.active_tasks_list
                    .array
                    .push(self.completed_tasks_list.array[val].clone());
                self.completed_tasks_list.array.remove(val);
                self.selected_project.completed_tasks = self.completed_tasks_list.array.clone();
                self.selected_project.active_tasks = self.active_tasks_list.array.clone();
                match self
                    .selected_project
                    .write_project_full_path(self.working_path.clone())
                {
                    Ok(_) => {}
                    Err(e) => self.create_message_popup(e.to_string()),
                };
            }
            None => {}
        }
    }

    fn create_message_popup(&mut self, message: String) {
        self.message_popup = MessageWindow::new(message);
    }

    fn update_project(&mut self) {
        self.selected_project = match load_project_from_path(self.working_path.clone()) {
            Ok(updated_project) => updated_project,
            Err(e) => {
                self.create_message_popup(e.to_string());
                Project::default()
            }
        };
        self.active_tasks_list = DisplayList::from(self.selected_project.active_tasks.clone());
        self.completed_tasks_list =
            DisplayList::from(self.selected_project.completed_tasks.clone());
    }
}

impl Service for TaskService {
    fn set_working_directory(&mut self, path: PathBuf) {
        self.working_path = path;
    }
}

impl Drawable for TaskService {
    fn display(&self, frame: &mut Frame<CrosstermBackend<Stdout>>, layout: Rect) {
        let initial_layout = Layout::default()
            .direction(Vertical)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(layout);

        // upper layout
        let task_layout = Layout::default()
            .direction(Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(initial_layout[0]);

        let active_task_block = Block::default().borders(Borders::ALL).title("Active Tasks");

        let active_task_display_list = List::new::<Vec<ListItem>>(
            self.active_tasks_list
                .array
                .clone()
                .into_iter()
                .map(|task| {
                    ListItem::new(Text::from(utils::wrap(
                        task.name,
                        task_layout[0].width as u32,
                    )))
                })
                .collect(),
        )
        .block(active_task_block)
        .highlight_style(
            tui::style::Style::default()
                .bg(tui::style::Color::Green)
                .add_modifier(tui::style::Modifier::BOLD),
        )
        .highlight_symbol("-> ");

        let completed_task_block = Block::default()
            .borders(Borders::ALL)
            .title("Completed Tasks");
        let completed_task_display_list = List::new::<Vec<ListItem>>(
            self.completed_tasks_list
                .array
                .clone()
                .into_iter()
                .map(|task| {
                    ListItem::new(Text::from(utils::wrap(
                        task.name,
                        task_layout[1].width as u32,
                    )))
                })
                .collect(),
        )
        .block(completed_task_block)
        .highlight_style(
            tui::style::Style::default()
                .bg(tui::style::Color::Green)
                .add_modifier(tui::style::Modifier::BOLD),
        )
        .highlight_symbol("-> ");
        if self.focused_on_active {
            frame.render_stateful_widget(
                active_task_display_list,
                task_layout[0],
                &mut self.active_tasks_list.state.clone(),
            );
            frame.render_widget(completed_task_display_list, task_layout[1]);
        } else {
            frame.render_widget(active_task_display_list, task_layout[0]);
            frame.render_stateful_widget(
                completed_task_display_list,
                task_layout[1],
                &mut self.completed_tasks_list.state.clone(),
            );
        }
        // Lower layout
        let description_block = Block::default().title("Description").borders(Borders::ALL);
        let description: String = match self.focused_on_active {
            true => match self.active_tasks_list.state.selected() {
                Some(val) => self.active_tasks_list.array[val].description.clone(),
                None => String::from("No task selected"),
            },
            false => match self.completed_tasks_list.state.selected() {
                Some(val) => self.completed_tasks_list.array[val].description.clone(),
                None => String::from("No task selected"),
            },
        };
        let description_paragraph = Paragraph::new(Text::from(description))
            .block(description_block)
            .wrap(Wrap { trim: false });
        frame.render_widget(description_paragraph, initial_layout[1]);

        // Popups
        if self.input_popup.is_active() {
            self.input_popup.display(frame, layout);
        }
        if self.message_popup.is_active() {
            self.message_popup.display(frame, layout);
        }
    }
}

impl InputReceptor for TaskService {
    fn handle_input_key(&mut self, key_code: KeyCode) {
        match self.input_mode {
            InputMode::CommandMode => match key_code {
                KeyCode::Left => {
                    self.focused_on_active = true;
                }
                KeyCode::Right => {
                    self.focused_on_active = false;
                }
                KeyCode::Char('a') => {
                    self.add_task_command();
                }
                KeyCode::Char('c') => {
                    if self.focused_on_active {
                        self.mark_selected_task_as_completed();
                        self.update_project();
                    }
                }
                KeyCode::Char('u') => {
                    if !self.focused_on_active {
                        self.mark_selected_task_as_uncompleted();
                        self.update_project();
                    }
                }
                KeyCode::Char('e') => {
                    self.edit_task_description();
                }
                KeyCode::Up => {
                    if self.focused_on_active {
                        self.active_tasks_list.previous();
                    } else {
                        self.completed_tasks_list.previous();
                    }
                }
                KeyCode::Down => {
                    if self.focused_on_active {
                        self.active_tasks_list.next();
                    } else {
                        self.completed_tasks_list.next();
                    }
                }
                _ => {}
            },
            InputMode::WriteMode => {
                match key_code {
                    _ => {
                        if self.message_popup.is_active() {
                            self.message_popup.handle_input_key(key_code);
                            if self.message_popup.is_completed() {
                                self.message_popup.set_active(false);
                                return;
                            }
                        }
                        self.input_popup.handle_input_key(key_code);
                        if !self.input_popup.is_active() {
                            self.input_mode = CommandMode;
                            return;
                        }
                        if self.input_popup.is_completed() {
                            match self.input_popup_type {
                                AddName => {
                                    self.selected_project.add_task(
                                        self.input_popup.get_input_data(),
                                        String::from("Description"),
                                    );
                                    match self
                                        .selected_project
                                        .write_project_full_path(self.working_path.clone())
                                    {
                                        Ok(_) => {
                                            self.input_popup.set_active(false);
                                            self.input_mode = CommandMode;
                                            self.update_project();
                                        }
                                        Err(e) => {
                                            self.create_message_popup(e.to_string());
                                        }
                                    };
                                }
                                EditDescription => {
                                    let inputted_string = self.input_popup.get_input_data();
                                    match self.focused_on_active {
                                        true => {
                                            match self.active_tasks_list.state.selected() {
                                                Some(val) => {
                                                    self.active_tasks_list.array[val].description =
                                                        inputted_string;
                                                    self.selected_project.active_tasks =
                                                        self.active_tasks_list.array.clone();
                                                    match self
                                                        .selected_project
                                                        .write_project_full_path(
                                                            self.working_path.clone(),
                                                        ) {
                                                        Ok(_) => {
                                                            self.input_popup.set_active(false);
                                                            self.input_mode = CommandMode;
                                                            self.update_project();
                                                        }
                                                        Err(e) => {
                                                            self.create_message_popup(
                                                                e.to_string(),
                                                            );
                                                        }
                                                    }
                                                }
                                                None => {
                                                    self.create_message_popup(String::from(
                                                        "Selected task is invalid",
                                                    ));
                                                }
                                            };
                                        }
                                        false => {
                                            match self.completed_tasks_list.state.selected() {
                                                Some(val) => {
                                                    self.completed_tasks_list.array[val]
                                                        .description = inputted_string;
                                                    self.selected_project.completed_tasks =
                                                        self.completed_tasks_list.array.clone();
                                                    match self
                                                        .selected_project
                                                        .write_project_full_path(
                                                            self.working_path.clone(),
                                                        ) {
                                                        Ok(_) => {
                                                            self.input_popup.set_active(false);
                                                            self.input_mode = CommandMode;
                                                            self.update_project();
                                                        }
                                                        Err(e) => {
                                                            self.create_message_popup(
                                                                e.to_string(),
                                                            );
                                                        }
                                                    }
                                                }
                                                None => {
                                                    self.create_message_popup(String::from(
                                                        "Selected task is invalid",
                                                    ));
                                                }
                                            };
                                        }
                                    };
                                }
                            }
                        }
                    }
                };
            }
            _ => {}
        };
    }

    fn get_controls_description(&self) -> String {
        if self.message_popup.is_active() {
            return self.message_popup.get_controls_description();
        } else if self.input_popup.is_active() {
            return self.input_popup.get_controls_description();
        } else {
            String::from("Navigate with arrows | C: Mark as completed | U: Mark as incomplete | A: Add task | E: Edit task description | Tab: Back To Projects")
        }
    }

    fn get_input_mode(&self) -> InputMode {
        match self.input_mode {
            InputMode::CommandMode => InputMode::CommandMode,
            InputMode::WriteMode => InputMode::WriteMode,
            InputMode::SubWindowInputs => InputMode::SubWindowInputs,
        }
    }
}
