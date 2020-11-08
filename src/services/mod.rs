pub mod project_service;
pub mod task_service;

pub trait Service {
    fn set_working_directory(&mut self, path: std::path::PathBuf);
}