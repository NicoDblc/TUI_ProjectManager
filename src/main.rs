mod structure;
use crossterm::event::{self, Event as CEvent, KeyCode};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

mod ui;
mod utils;
mod services;

enum Event<I> {
    Input(I),
    Tick,
}

fn main() {
    // TODO:      - handle path pass as parameter to the program
    utils::create_working_folder_if_not_exist();
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

    let mut app = structure::Application::new(utils::get_working_folder());
    while app.is_running {
        app.update();
        match rx.recv().unwrap() {
            Event::Input(event) => match event.code {
                _ => app.handle_inputs(event.code),
            },
            Event::Tick => {}
        }
    }
}
