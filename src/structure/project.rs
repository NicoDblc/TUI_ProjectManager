use crate::structure::task::Task;
use crate::structure::TaskContainer;
use crate::utils::PROJECT_FILE_EXTENSION;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::Error;
use std::path::PathBuf;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub description: String,
    pub active_tasks: Vec<Task>,
    pub completed_tasks: Vec<Task>,
}

impl Project {
    pub fn new(project_name: String) -> Project {
        Project {
            name: project_name,
            description: String::from("Sample description"),
            active_tasks: vec![],
            completed_tasks: vec![],
        }
    }

    pub fn write_project_full_path(&self, path_for_project: PathBuf) -> Result<(), std::io::Error> {
        let project_string = match serde_json::to_string(self) {
            Ok(p_string) => p_string,
            Err(e) => {
                return Result::Err(std::io::Error::from(e));
            }
        };
        match std::fs::write(path_for_project.clone(), project_string) {
            Ok(()) => Ok(()),
            Err(e) => {
                println!("Writing error: {}", path_for_project.to_str().unwrap());
                return Result::Err(e);
            }
        }
    }
}

impl TaskContainer for Project {
    fn add_task(&mut self, task_name: String, task_description: String) {
        let task = Task::new(task_name, task_description);
        self.active_tasks.push(task);
    }
}

pub fn load_project_from_path(path: PathBuf) -> Result<Project, std::io::Error> {
    match std::fs::read_to_string(path) {
        Ok(project_string) => match serde_json::from_str(project_string.as_str()) {
            Ok(deserialized_project) => Result::Ok(deserialized_project),
            Err(e) => Result::Err(Error::from(e)),
        },
        Err(e) => Result::Err(e),
    }
}

pub fn get_projects_in_path(path: PathBuf) -> Vec<Project> {
    let mut serialized_projects: Vec<Project> = vec![];
    let folder_result = std::fs::read_dir(path.as_path()).unwrap();
    for file in folder_result {
        let f = file.unwrap();
        if f.file_type().unwrap().is_file() {
            match f.path().extension() {
                Some(ext) => {
                    if ext == PROJECT_FILE_EXTENSION {
                        match match serde_json::from_str(
                            std::fs::read_to_string(f.path()).unwrap().as_str(),
                        ) {
                            Ok(result) => Some(result),
                            Err(_) => None,
                        } {
                            Some(project) => serialized_projects.push(project),
                            _ => {}
                        }
                    }
                }
                _ => {}
            };
        }
    }
    serialized_projects
}
