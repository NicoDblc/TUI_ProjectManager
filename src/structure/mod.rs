pub mod application;
pub mod project;
pub mod task;

trait InformationDisplay {
    fn get_description(&self) -> String;
    fn get_name(&self) -> String;
}

trait Completable {
    fn complete(&self);
}

pub trait TaskContainer {
    fn add_task(&mut self, task_name: String, task_description: String);
}
