use crate::services::task_service::TaskInputChoice::{AddName, EditDescription};
use crate::services::Service;
use crate::structure::{Project, Task};
use crate::ui::InputMode::CommandMode;
use crate::ui::PopupInputWindow;
use crate::ui::{Completable, DisplayList, Drawable, InputMode, InputReceptor, InputReturn};
use crossterm::event::KeyCode;
use std::io::{Error, Stdout};
use std::path::PathBuf;
use tui::backend::CrosstermBackend;
use tui::layout::Direction::{Horizontal, Vertical};
use tui::layout::{Constraint, Layout, Rect};
use tui::text::Text;
use tui::widgets::{Block, Borders, List, ListItem};
use tui::Frame;

enum TaskInputChoice {
    AddName,
    EditDescription,
}

struct TaskService {
    working_path: PathBuf,
    selected_project: Project,
    active_tasks_list: DisplayList<Task>,
    completed_tasks_list: DisplayList<Task>,
    focused_on_active: bool,
    input_mode: InputMode,
    input_popup: PopupInputWindow,
    input_popup_type: TaskInputChoice,
}

impl TaskService {
    fn add_task_command(&mut self) {
        self.input_popup_type = AddName;
        self.input_mode = InputMode::WriteMode;
        self.input_popup = PopupInputWindow::new(String::from("Enter Task Name"));
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
        self.input_popup = PopupInputWindow::new(String::from("Edit taks description"));
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
                self.selected_project
                    .write_project_to_path(self.working_path.clone());
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
                self.selected_project
                    .write_project_to_path(self.working_path.clone());
            }
            None => {}
        }
    }
}

impl Service for TaskService {
    fn set_working_directory(&mut self, path: PathBuf) {
        self.working_path = path;
    }
}

impl Drawable for TaskService {
    // TODO: display the Popups
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
                .map(|task| ListItem::new(Text::from(task.name)))
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
                .map(|task| ListItem::new(Text::from(task.name)))
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
    }
}

impl InputReceptor for TaskService {
    fn handle_input_key(&mut self, key_code: KeyCode) {
        //TODO: handle write mode
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
                    }
                }
                KeyCode::Char('u') => {
                    if !self.focused_on_active {
                        self.mark_selected_task_as_uncompleted();
                    }
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
                        self.input_popup.handle_input_key(key_code);
                        if !self.input_popup.is_active() {
                            self.input_mode = CommandMode;
                            return;
                        }
                        if self.input_popup.is_completed() {
                            match self.input_popup_type {
                                AddName => {}
                                EditDescription => match self.focused_on_active {
                                    true => {


                                        // Update project
                                    }
                                    false => {


                                    }
                                },
                            }
                        }
                    }
                };
            }
        };
    }

    fn get_controls_description(&self) -> String {
        unimplemented!()
    }

    fn get_input_mode(&self) -> InputMode {
        unimplemented!()
    }
}
