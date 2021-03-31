pub static PROJECT_FILE_EXTENSION: &str = "pman";
use crate::structure::project::Project;
use crate::structure::task::Task;
use crate::structure::*;
use std::io::Error;
use std::ops::Add;
use std::path::{Path, PathBuf};

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
    let shortened_wrap = wrap_at_length - 4; // arrow length
    for word in words.iter() {
        line_length += word.chars().count() as u32;
        line_length += 1; // accounting for the space
        if line_length >= shortened_wrap {
            final_string = final_string.add("\n");
            line_length = word.chars().count() as u32;
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
