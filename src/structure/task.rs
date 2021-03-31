use crate::structure::{InformationDisplay, TaskContainer};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Task {
    pub name: String,
    pub description: String,
    pub time_spent: i32,
    pub estimate: i32,
    pub sub_tasks: Vec<Task>,
    pub tags: Vec<String>,
}

impl Task {
    pub fn new(task_name: String, task_description: String) -> Task {
        Task {
            name: task_name,
            description: task_description,
            time_spent: 0,
            estimate: 0,
            sub_tasks: vec![],
            tags: vec![],
        }
    }
}

impl InformationDisplay for Task {
    fn get_description(&self) -> String {
        self.description.clone()
    }

    fn get_name(&self) -> String {
        self.description.clone()
    }
}

impl TaskContainer for Task {
    fn add_task(&mut self, task_name: String, task_description: String) {
        let task = Task {
            name: task_name,
            description: task_description,
            tags: vec![],
            time_spent: 0,
            estimate: 0,
            sub_tasks: vec![],
        };
        self.sub_tasks.push(task);
    }
}
