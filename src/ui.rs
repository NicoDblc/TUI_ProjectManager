use serde::Deserialize;
use serde::Serialize;

use std::io;
use tui::backend::CrosstermBackend;
use tui::{Frame, Terminal};

use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::text::Text;
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};

use crate::structure::Project;
use crate::utils;
use std::io::Stdout;
use crossterm::event::KeyCode;
use serde_json;

#[derive(Default)]
struct DisplayList<T> {
    state: ListState,
    array: Vec<T>,
}

impl<T> DisplayList<T> {
    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i < self.array.len() - 1 {
                    i + 1
                } else {
                    i
                }
            }
            None => 0,
        };
        if self.array.len() > 0 {
            self.state.select(Some(i));
        }
    }
    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i > 0 {
                    i - 1
                } else {
                    0
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn from(content: Vec<T>) -> DisplayList<T> {
        let content_len = content.len();
        let mut dl = DisplayList {
            state: Default::default(),
            array: content,
        };
        if content_len > 0 {
            dl.state.select(Some(0));
        }
        dl
    }
}

pub trait Window {
    fn display(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, layout: Rect);
    fn get_controls_description(&self) -> String;
    fn handle_input_key(&mut self, key_code: KeyCode);
    fn input_up(&mut self);
    fn input_down(&mut self);

    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_y) / 2),
                    Constraint::Percentage(percent_y),
                    Constraint::Percentage((100 - percent_y) / 2),
                ]
                    .as_ref(),
            )
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_x) / 2),
                    Constraint::Percentage(percent_x),
                    Constraint::Percentage((100 - percent_x) / 2),
                ]
                    .as_ref(),
            )
            .split(popup_layout[1])[1]
    }

}

pub struct ProjectWindow<'a> {
    // Everything that is contained in the draw call for the main window
    projects_to_display: DisplayList<Project>,
    selected_project_active_tasks: Vec<ListItem<'a>>,
    selected_project_completed_tasks: Vec<ListItem<'a>>,
}

impl<'a> ProjectWindow<'a> {
    pub fn new(projects: Vec<Project>) -> ProjectWindow<'a> {
        let mut pw = ProjectWindow {
            projects_to_display: DisplayList::from(projects),
            selected_project_active_tasks: Vec::new(),
            selected_project_completed_tasks: Vec::new(),
        };
        if pw.projects_to_display.array.len() > 0 {
            pw.update_project_selection();
        }
        pw
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
    fn draw_project_addition_popup(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, layout: Rect) {
        let popup_layout:Rect = <ProjectWindow as Window>::centered_rect(50,50, layout);

    }

    fn add_project_request(&mut self) {
        // TODO: popup to add name and description to project
    }

    fn delete_selected_project(&mut self) {
        // TODO: delete selected project
    }

    fn edit_selected_project_name(&mut self) {
        // TODO: Implement (with pop up)
    }

    fn edit_selected_project_description(&mut self) {
        // TODO: Implement (with pop up)
    }
}

impl<'a> Window for ProjectWindow<'a> {
    fn display(&mut self, frame: &mut Frame<CrosstermBackend<Stdout>>, layout: Rect) {
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
            &mut self.projects_to_display.state,
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
    }

    fn get_controls_description(&self) -> String {
        String::from("a - Add project     d - Delete project     e - Edit Project Description     n - Edit Project name")
    }

    fn handle_input_key(&mut self, key_code: KeyCode){
        match key_code {
            KeyCode::Up => {
                self.input_up();
            }
            KeyCode::Down => {
                self.input_down();
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
        }
    }

    fn input_up(&mut self) {
        self.projects_to_display.previous();
        self.update_project_selection();
    }

    fn input_down(&mut self) {
        self.projects_to_display.next();
        self.update_project_selection();
    }
}
