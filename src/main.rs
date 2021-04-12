mod structure;
use crossterm::event::{self, Event as CEvent, KeyModifiers};
use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

#[macro_use]
extern crate smart_default;

mod popups;
mod services;
mod ui;
mod utils;

enum Event<I> {
    Input(I),
    Tick,
}

fn main() {
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

    let mut app = structure::application::Application::new(utils::get_working_folder());
    while app.is_running {
        match rx.recv().unwrap() {
            Event::Input(event) => {
                app.handle_inputs(event)
            },
            Event::Tick => {
                app.update();
            }
        }
    }
}
