pub static PROJECT_FILE_EXTENSION: &str = "pman";
use std::ops::Add;
use std::path::{Path, PathBuf};

use crate::structure::{Project, TaskContainer};

// TODO: Create working folder if it does not exist
// TODO: Get working folder

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
    let home_path = dirs::home_dir().unwrap();
    let folder_path = String::from('.').add(PROJECT_FILE_EXTENSION);
    home_path.join(Path::new(folder_path.as_str()))
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
    let project_string = serde_json::to_string(&p).unwrap();
    let work_folder = get_working_folder();
    std::fs::write(
        work_folder.join(name.add(".pman")).as_path(),
        project_string,
    ).unwrap();
}
