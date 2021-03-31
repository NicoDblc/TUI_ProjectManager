pub mod project_s;
pub mod task_s;

pub trait Service {
    fn set_working_directory(&mut self, path: std::path::PathBuf);
}
