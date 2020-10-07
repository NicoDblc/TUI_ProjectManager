mod structure;
use std::path;

fn main() {
    // let args: Vec<String> = std::env::args().collect();
    // for a in args {
    //     println!("{}", a);
    // }

    let home_path = dirs::home_dir().unwrap();
    let mut app = structure::Application::new(home_path);
    while app.is_running {
        // TODO: handle inputs
        app.update();
    }
}
