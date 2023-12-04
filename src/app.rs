use std::{
    error::Error,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode};

pub struct Timer {
    pub length: Duration,
    pub time_left: Duration,
    last_tick: Option<Instant>,
    pub running: bool,
    pub title: String,
}

impl Timer {
    pub fn new(length: Duration) -> Timer {
        Timer {
            length,
            time_left: length,
            last_tick: None,
            running: false,
            title: "New Timer".to_string(),
        }
    }
}

// pub struct TimerBox {
//     pub timer: Timer,
//     pub title: String,
// }
//
// impl TimerBox {
//     pub fn new(timer: Timer) -> TimerBox {
//         TimerBox {
//             timer,
//             title: "New Timer".to_string(),
//         }
//     }
// }

pub struct App {
    pub timers: Vec<Timer>,
    pub selected_timer: Option<usize>,
    alarming: bool,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> App {
        App {
            timers: Vec::new(),
            selected_timer: None,
            alarming: false,
            should_quit: false,
        }
    }

    pub fn update_timers(&mut self) {
        for timer in &mut self.timers {
            let now = Instant::now();
            // let t = &mut tb.timer;

            if timer.running {
                if let Some(tick) = timer.last_tick {
                    let time_elapsed = now.duration_since(tick);

                    if let Some(new_time_left) = timer.time_left.checked_sub(time_elapsed) {
                        timer.time_left = new_time_left;
                    } else {
                        timer.time_left = Duration::ZERO;
                        timer.running = false;
                        self.alarming = true;
                    }
                } else {
                    timer.last_tick = Some(now);
                }
            }
        }
    }

    pub fn add_timer(&mut self) {
        let new_timer = Timer::new(Duration::ZERO);

        // let new_timer_box = TimerBox::new(new_timer);

        self.timers.push(new_timer);

        self.selected_timer = match self.selected_timer {
            Some(i) => Some(i),
            None => Some(0),
        }
    }

    fn delete_timer(&mut self) {
        match self.selected_timer {
            None => {}
            Some(i) => {
                self.timers.remove(i);

                self.selected_timer = {
                    let s = self.timers.len();

                    if s == 0 {
                        None
                    } else {
                        if s == i {
                            Some(s - 1)
                        } else {
                            Some(i)
                        }
                    }
                }
            }
        }
    }

    fn increment_selection(&mut self) {
        self.selected_timer = match self.selected_timer {
            Some(i) => {
                let mut n = i + 1;

                if n >= self.timers.len() {
                    n = 0;
                }

                Some(n)
            }
            None => None,
        }
    }

    fn decrement_selection(&mut self) {
        self.selected_timer = match self.selected_timer {
            Some(i) => match i {
                0 => Some(self.timers.len() - 1),
                _ => Some(i - 1),
            },
            None => None,
        }
    }

    fn start_timer(&mut self) {
        match self.selected_timer {
            Some(i) => self.timers[i].running = true,
            None => (),
        }
    }

    fn stop_timer(&mut self) {
        match self.selected_timer {
            Some(i) => self.timers[i].running = false,
            None => (),
        }
    }

    fn toggle_timer(&mut self) {
        match self.selected_timer {
            Some(i) => self.timers[i].running = !self.timers[i].running,
            None => (),
        }
    }

    pub fn handle_events(&mut self) -> Result<(), Box<dyn Error>> {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => self.should_quit = true,
                        KeyCode::Char('Q') => self.should_quit = true,
                        KeyCode::Char('k') => self.decrement_selection(),
                        KeyCode::Char('j') => self.increment_selection(),
                        KeyCode::Char('a') => self.add_timer(),
                        KeyCode::Char('d') => self.delete_timer(),
                        _ => (),
                    }
                }
            }
        }
        Ok(())
    }
}
