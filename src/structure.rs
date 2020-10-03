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

trait TaskContainer {
    fn add_task(task_description: String);
}

struct Project {
    name: String,
    description: String,
    tasks: Vec<Task>,
}

impl TaskContainer for Project {}

struct Task {
    description: String,
    time_spent: i32,
    estimate: i32,
    sub_tassk: Vec<Task>,
}

impl TaskContainer for Task {}
