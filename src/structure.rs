pub struct Folder {
    active_projects: Vec<Project>,
    archive_projects: Vec<Project>,
}

trait Inform {
    fn display_information(&self);
}

trait Completable {
    fn complete(&self);
}

struct Project {
    name: String,
    description: String,
    tasks: Vec<Task>,
}

impl Project {
    pub fn add_task(task_description: String) {}
}

struct Task {
    description: String,
    time_spent: i32,
    estimate: i32,
}

impl
