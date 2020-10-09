mod structure;
use crossterm::event::{self, Event as CEvent, KeyCode};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

enum Event<I> {
    Input(I),
    Tick,
}

fn main() {
    // let args: Vec<String> = std::env::args().collect();
    // for a in args {
    //     println!("{}", a);
    // }

    let home_path = dirs::home_dir().unwrap();
    let (tx, rx) = mpsc::channel();

    let tick_rate = Duration::from_millis(250);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });

    let mut app = structure::Application::new(home_path);
    while app.is_running {
        app.update();
        match rx.recv().unwrap() {
            Event::Input(event) => match event.code {
                // KeyCode::Char(c) => app.on_key(c),
                // KeyCode::Left => app.on_left(),
                KeyCode::Up => app.press_up(),
                // KeyCode::Right => app.on_right(),
                KeyCode::Down => app.press_down(),
                _ => {}
            },
            Event::Tick => {}
        }
    }
}
