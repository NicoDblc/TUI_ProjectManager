pub static PROJECT_FILE_EXTENSION: &str = "pman";
use std::ops::Add;
use std::path::{Path, PathBuf};

use crate::structure::{Project, Task, TaskContainer};
use std::io::Error;

pub fn create_working_folder_if_not_exist() {
    let working_folder = get_working_folder();
    if !working_folder.exists() {
        match std::fs::create_dir(working_folder.as_path()) {
            Ok(_) => {}
            Err(e) => {
                panic!("Error occured while create the working dir: {}", e);
            }
        };
    }
}

pub fn get_working_folder() -> PathBuf {
    // TODO: check if program was opened in a specific directory
    let home_path = dirs::home_dir().unwrap();
    let folder_path = String::from('.').add(PROJECT_FILE_EXTENSION);
    home_path.join(Path::new(folder_path.as_str()))
}

pub fn delete_project_of_name(project_name: String, working_path: PathBuf) -> Result<(), Error> {
    let mut path = working_path.join(project_name);
    path.set_extension(PROJECT_FILE_EXTENSION);
    match std::fs::remove_file(path.as_path()) {
        Ok(()) => Ok(()),
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

#[test]
fn create_dummy_project() {
    create_dummy_project_with_name("project1".to_string());
    create_dummy_project_with_name("project2".to_string());
    create_dummy_project_with_name("project3".to_string());
}

fn create_dummy_project_with_name(name: String) {
    create_working_folder_if_not_exist();
    let mut p = Project::new(name.clone());
    p.description = p.name.clone().add(" description");
    p.add_task(name.clone().add("Jambalaya 1"));
    p.add_task(name.clone().add("Jambalaya 2"));
    p.completed_tasks
        .push(Task::new(String::from("a completed task")));
    let project_string = serde_json::to_string(&p).unwrap();
    let work_folder = get_working_folder();
    std::fs::write(
        work_folder.join(name.add(".pman")).as_path(),
        project_string,
    )
    .unwrap();
}
