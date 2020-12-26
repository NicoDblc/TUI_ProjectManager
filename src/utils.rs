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
                panic!("Error occurred while creating the working dir: {}", e);
            }
        };
    }
}

pub fn wrap(string_to_wrap: String, wrap_at_length: u32) -> String {
    let words: Vec<&str> = string_to_wrap.split(" ").collect();
    let mut line_length: u32 = 0;
    let mut final_string = String::new();
    for word in words.iter() {
        line_length += word.len() as u32;
        if line_length >= wrap_at_length {
            final_string = final_string.add("\n");
            line_length = 0;
        } else {
            final_string = final_string.add(" ");
        }
        final_string = final_string.add(word);
    }
    final_string
}

pub fn get_working_folder() -> PathBuf {
    let work_path = match std::env::args().nth(1) {
        Some(val) => std::path::PathBuf::from(val),
        None => dirs::home_dir().unwrap(),
    };
    let folder_path = String::from('.').add(PROJECT_FILE_EXTENSION);
    work_path.join(Path::new(folder_path.as_str()))
}

pub fn delete_project_of_name(project_name: String, working_path: PathBuf) -> Result<(), Error> {
    let mut path = working_path.join(project_name);
    path.set_extension(PROJECT_FILE_EXTENSION);
    match std::fs::remove_file(path.as_path()) {
        Ok(()) => Ok(()),
        Err(e) => Result::Err(e),
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

#[test]
fn create_dummy_project() {
    create_dummy_project_with_name(String::from("project1"));
    create_dummy_project_with_name(String::from("project2"));
    create_dummy_project_with_name(String::from("project3"));
}
#[allow(dead_code)]
fn create_dummy_project_with_name(name: String) {
    create_working_folder_if_not_exist();
    let mut p = Project::new(name.clone());
    p.description = p.name.clone().add(" description");
    p.add_task(
        name.clone().add("test 1"),
        String::from("Sample description"),
    );
    p.add_task(
        name.clone().add("test2 2"),
        String::from("Sample description"),
    );
    p.completed_tasks.push(Task::new(
        String::from("a completed task"),
        String::from("Sample description"),
    ));
    let mut project_file_path = get_working_folder().join(p.name.clone());
    project_file_path.set_extension(PROJECT_FILE_EXTENSION);
    match p.write_project_full_path(project_file_path) {
        Ok(_) => {}
        Err(e) => panic!("{}", e),
    }
}
