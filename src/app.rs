use std::{
    collections::HashMap,
    error::Error,
    fmt::Write,
    fs::File,
    hash::Hash,
    io::BufReader,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode};
use lazy_static::lazy_static;
use rodio::{source::SineWave, Decoder, OutputStream, OutputStreamHandle, Sink, Source};
use serde::{Deserialize, Serialize};
use tui_textarea::{CursorMove, TextArea};

// const EDITFIELD_NEXT: HashMap<EditField, EditField> = HashMap::from();

lazy_static! {
    static ref EDITFIELD_NEXT: HashMap<EditField, EditField> = HashMap::from([
        (EditField::Description, EditField::Hours1),
        (EditField::Hours1, EditField::Hours2),
        (EditField::Hours2, EditField::Hours3),
        (EditField::Hours3, EditField::Minutes1),
        (EditField::Minutes1, EditField::Minutes2),
        (EditField::Minutes2, EditField::Seconds1),
        (EditField::Seconds1, EditField::Seconds2),
        (EditField::Seconds2, EditField::Description),
    ]);
    static ref EDITFIELD_PREVIOUS: HashMap<EditField, EditField> = HashMap::from([
        (EditField::Description, EditField::Seconds2),
        (EditField::Hours1, EditField::Description),
        (EditField::Hours2, EditField::Hours1),
        (EditField::Hours3, EditField::Hours2),
        (EditField::Minutes1, EditField::Hours3),
        (EditField::Minutes2, EditField::Minutes1),
        (EditField::Seconds1, EditField::Minutes2),
        (EditField::Seconds2, EditField::Seconds1),
    ]);
    // static ref EDITFIELD_HOUR: HashMap<EditField, EditField> = HashMap::from([
    //     (EditField::Hours1, EditField::Hours2),
    //     (EditField::Hours2, EditField::Hours3),
    //     (EditField::Hours3, EditField::Hours1),
    //     (EditField::Minutes1, EditField::Hours1),
    //     (EditField::Minutes2, EditField::Hours1),
    //     (EditField::Seconds1, EditField::Hours1),
    //     (EditField::Seconds2, EditField::Hours1),
    // ]);
    // static ref EDITFIELD_MINUTE: HashMap<EditField, EditField> = HashMap::from([
    //     (EditField::Hours1, EditField::Minutes1),
    //     (EditField::Hours2, EditField::Minutes1),
    //     (EditField::Hours3, EditField::Minutes1),
    //     (EditField::Minutes1, EditField::Minutes2),
    //     (EditField::Minutes2, EditField::Minutes1),
    //     (EditField::Seconds1, EditField::Minutes1),
    //     (EditField::Seconds2, EditField::Minutes1),
    // ]);
    // static ref EDITFIELD_SECOND: HashMap<EditField, EditField> = HashMap::from([
    //     (EditField::Hours1, EditField::Seconds1),
    //     (EditField::Hours2, EditField::Seconds1),
    //     (EditField::Hours3, EditField::Seconds1),
    //     (EditField::Minutes1, EditField::Seconds1),
    //     (EditField::Minutes2, EditField::Seconds1),
    //     (EditField::Seconds1, EditField::Seconds2),
    //     (EditField::Seconds2, EditField::Seconds1),
    // ]);
}

#[derive(Serialize, Deserialize, Debug)]
pub enum TimerState {
    Stopped,
    Running,
    Alarming,
}

#[derive(Serialize, Deserialize, Debug)]
struct SerializeableTimerParts {
    length: Duration,
    time_left: Duration,
    description: String,
}

impl SerializeableTimerParts {
    fn new(description: String, length: Duration, time_left: Duration) -> SerializeableTimerParts {
        SerializeableTimerParts {
            length,
            time_left,
            description,
        }
    }
}

// #[derive(Serialize, Deserialize, Debug)]
pub struct Timer {
    // pub length: Duration,
    // pub time_left: Duration,
    serializeable_parts: SerializeableTimerParts,
    last_started: Option<Instant>,
    time_left_at_last_tick: Duration,
    pub state: TimerState,
    // pub title: String,
}

impl Timer {
    pub fn new(description: String, length: Duration) -> Timer {
        Timer {
            // length,
            // time_left: length,
            serializeable_parts: SerializeableTimerParts::new(description, length, length),
            last_started: None,
            time_left_at_last_tick: length,
            state: TimerState::Stopped,
            // title: description,
        }
    }

    pub fn default() -> Timer {
        let default_duration = Duration::from_secs(300);
        Timer {
            // length: default_duration,
            // time_left: default_duration,
            serializeable_parts: SerializeableTimerParts::new(
                String::from("New Timer"),
                default_duration,
                default_duration,
            ),
            last_started: None,
            time_left_at_last_tick: default_duration,
            state: TimerState::Stopped,
            // title: "New Timer".to_string(),
        }
    }

    fn from_serializeable(parts: SerializeableTimerParts) -> Timer {
        let time_left_at_last_tick = parts.time_left;

        Timer {
            serializeable_parts: parts,
            last_started: None,
            time_left_at_last_tick,
            state: TimerState::Stopped,
        }
    }

    pub fn clone_description(&self) -> String {
        self.serializeable_parts.description.clone()
    }

    pub fn get_length(&self) -> Duration {
        self.serializeable_parts.length
    }

    pub fn get_time_left(&self) -> Duration {
        self.serializeable_parts.time_left
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum EditField {
    Description,
    Hours1,
    Hours2,
    Hours3,
    Minutes1,
    Minutes2,
    Seconds1,
    Seconds2,
}

pub struct EditValue {
    value: i64,
    modulus: i64,
}

impl EditValue {
    fn new(value: i64, modulus: i64) -> EditValue {
        EditValue { value, modulus }
    }

    // fn default() -> EditValue {
    //     EditValue {
    //         value: 0,
    //         modulus: 10,
    //     }
    // }

    fn set_value(&mut self, value: i64) {
        if value < self.modulus {
            self.value = value;
        } else {
            self.value = self.modulus - 1;
        }
    }

    fn inc_value(&mut self) {
        self.value = (self.value + 1) % self.modulus;
    }

    fn dec_value(&mut self) {
        self.value = (self.value + self.modulus - 1) % self.modulus;
    }

    pub fn value_as_string(&self) -> String {
        format!("{}", self.value)
    }
}

pub struct EditValues<'a> {
    pub descript: TextArea<'a>,
    pub hours1: EditValue,
    pub hours2: EditValue,
    pub hours3: EditValue,
    pub minutes1: EditValue,
    pub minutes2: EditValue,
    pub seconds1: EditValue,
    pub seconds2: EditValue,
}

impl EditValues<'_> {
    fn new(descript: String, length: Duration) -> EditValues<'static> {
        let mut descript = TextArea::new(descript.lines().map(|s| s.to_owned()).collect());

        descript.move_cursor(CursorMove::Bottom);
        descript.move_cursor(CursorMove::End);

        // let hours = length.as_secs() / 3600;
        // let minutes = (length.as_secs() % 3600) / 60;
        // let seconds = length.as_secs() % 60;

        // Duration shouldn't be large enough to panic - double check this
        // if you serde the alarms later! todo!()
        let hours1: i64 = (length.as_secs() / 3600).try_into().unwrap();
        let hours2: i64 = (hours1 / 10) % 10;
        let hours3: i64 = hours1 % 10;
        let hours1 = hours1 / 100;

        let minutes1: i64 = ((length.as_secs() / 60) % 60).try_into().unwrap();
        let minutes2 = minutes1 % 10;
        let minutes1 = minutes1 / 10;

        let seconds1: i64 = (length.as_secs() % 60).try_into().unwrap();
        let seconds2: i64 = seconds1 % 10;
        let seconds1 = seconds1 / 10;

        EditValues {
            descript,
            hours1: EditValue::new(hours1, 10),
            hours2: EditValue::new(hours2, 10),
            hours3: EditValue::new(hours3, 10),
            minutes1: EditValue::new(minutes1, 6),
            minutes2: EditValue::new(minutes2, 10),
            seconds1: EditValue::new(seconds1, 6),
            seconds2: EditValue::new(seconds2, 10),
        }
    }

    fn default() -> EditValues<'static> {
        // let mut lines = Vec::new();
        //
        // lines.push(String::from("New Timer"));

        let lines = vec![String::from("New Timer")];

        let mut descript = TextArea::new(lines);

        descript.move_cursor(CursorMove::Bottom);
        descript.move_cursor(CursorMove::End);

        EditValues {
            descript,
            hours1: EditValue::new(0, 10),
            hours2: EditValue::new(0, 10),
            hours3: EditValue::new(0, 10),
            minutes1: EditValue::new(0, 6),
            minutes2: EditValue::new(5, 10),
            seconds1: EditValue::new(0, 6),
            seconds2: EditValue::new(0, 10),
        }
    }

    fn to_duration(&self) -> Duration {
        Duration::from_secs(
            ((100 * self.hours1.value + 10 * self.hours2.value + self.hours3.value) * 3600
                + (10 * self.minutes1.value + self.minutes2.value) * 60
                + 10 * self.seconds1.value
                + self.seconds2.value)
                .try_into()
                .unwrap(),
        )
    }

    fn get_field(&mut self, field: &EditField) -> &mut EditValue {
        match field {
            EditField::Hours1 => &mut self.hours1,
            EditField::Hours2 => &mut self.hours2,
            EditField::Hours3 => &mut self.hours3,
            EditField::Minutes1 => &mut self.minutes1,
            EditField::Minutes2 => &mut self.minutes2,
            EditField::Seconds1 => &mut self.seconds1,
            EditField::Seconds2 => &mut self.seconds2,
            EditField::Description => panic!("Tried to get the description as an edit field"),
        }
    }
}

pub enum ErrorType {
    SoundDevice,
    File,
}

pub enum AppScreen {
    Main,
    Editing(EditField),
    Navigating,
    Error(ErrorType),
}

struct AlarmCounter {
    alarming_timers: u32,
    _stream: Option<OutputStream>,
    _stream_handle: Option<OutputStreamHandle>,
    alarm_sink: Option<Sink>,
}

impl AlarmCounter {
    fn new(filename: &str) -> AlarmCounter {
        // let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        //
        if let Ok((_stream, stream_handle)) = OutputStream::try_default() {
            if let Ok(sink) = Sink::try_new(&stream_handle) {
                if let Ok(file) = File::open(filename) {
                    let file = BufReader::new(file);

                    if let Ok(source) = Decoder::new(file) {
                        let source = source.repeat_infinite();

                        sink.append(source);
                        sink.pause();

                        return AlarmCounter {
                            alarming_timers: 0,
                            _stream: Some(_stream),
                            _stream_handle: Some(stream_handle),
                            alarm_sink: Some(sink),
                        };
                    }
                } else {
                    let source = SineWave::new(600.0)
                        .take_duration(Duration::from_millis(1000))
                        .delay(Duration::from_millis(1000))
                        .repeat_infinite()
                        .skip_duration(Duration::from_millis(1000));
                    // let source = from_iter(
                    //     SineWave::new(600.0)
                    //         .take_duration(Duration::from_millis(1000))
                    //         .chain(
                    //             SineWave::new(600.0)
                    //                 .amplify(0.0)
                    //                 .take_duration(Duration::from_millis(1000)),
                    //         ),
                    // )
                    // .repeat_infinite();

                    // let source = SineWave::new(600.0)
                    //     .take_duration(Duration::from_millis(1000))
                    //     .chain(SineWave::new(600.0).take_duration(Duration::from_millis(1000)))
                    //     .collect::<Source>()
                    //     // .delay(Duration::from_millis(1000))
                    //     .repeat_infinite();

                    sink.append(source);
                    sink.pause();

                    // sink.append(source2);

                    return AlarmCounter {
                        alarming_timers: 0,
                        _stream: Some(_stream),
                        _stream_handle: Some(stream_handle),
                        alarm_sink: Some(sink),
                    };
                }
            }
        }

        AlarmCounter {
            alarming_timers: 0,
            _stream: None,
            _stream_handle: None,
            alarm_sink: None,
        }

        // let sink = Sink::try_new(&stream_handle).unwrap();

        // let file = BufReader::new(File::open(filename).unwrap());

        // let source = Decoder::new(file).unwrap().repeat_infinite();

        // sink.append(source);
        // sink.pause();

        // AlarmCounter {
        //     alarming_timers: 0,
        //     _stream,
        //     _stream_handle: stream_handle,
        //     alarm_sink: sink,
        // }
    }

    fn increase_counter(&mut self) {
        if self.alarming_timers == 0 {
            // for _ in 1..=10 {
            //     println!("IT GETS HERE");
            // }
            if let Some(sink) = &self.alarm_sink {
                sink.play()
            }
        }

        self.alarming_timers += 1;
    }

    fn decrease_counter(&mut self) {
        self.alarming_timers -= 1;

        if self.alarming_timers == 0 {
            if let Some(sink) = &self.alarm_sink {
                sink.pause()
            }
        }
    }
}

pub struct App<'a> {
    pub timers: Vec<Timer>,
    pub selected_timer: Option<usize>,
    pub should_quit: bool,
    pub screen: AppScreen,
    pub edit_values: EditValues<'a>,
    pub successful_init: bool,
    alarm_counter: AlarmCounter,
}

impl App<'_> {
    // Singleton so this is fine
    pub fn new() -> App<'static> {
        let alarm_counter = AlarmCounter::new("rodio_alarm_test.flac");

        let successful_init = alarm_counter.alarm_sink.is_some();
        App {
            timers: Vec::new(),
            selected_timer: None,
            should_quit: false,
            screen: AppScreen::Main,
            edit_values: EditValues::default(),
            alarm_counter,
            successful_init,
        }
    }

    pub fn update_timers(&mut self) {
        for timer in &mut self.timers {
            let now = Instant::now();
            // let t = &mut tb.timer;

            match timer.state {
                TimerState::Stopped => (),
                TimerState::Running => {
                    if let Some(tick) = timer.last_started {
                        let time_elapsed = now.duration_since(tick);

                        if let Some(new_time_left) =
                            timer.time_left_at_last_tick.checked_sub(time_elapsed)
                        {
                            timer.serializeable_parts.time_left = new_time_left;
                        } else {
                            timer.serializeable_parts.time_left = Duration::ZERO;
                            timer.time_left_at_last_tick = Duration::ZERO;
                            timer.state = TimerState::Alarming;
                            // self.alarming_timers += 1;
                            self.alarm_counter.increase_counter();
                        }
                    } else {
                        timer.last_started = Some(now);
                    }
                }
                TimerState::Alarming => (),
            }
        }
    }

    fn add_default_timer(&mut self) {
        let new_timer = Timer::default();

        // let new_timer_box = TimerBox::new(new_timer);

        self.timers.push(new_timer);

        self.selected_timer = match self.selected_timer {
            Some(i) => Some(i),
            None => Some(0),
        }
    }

    fn add_timer_from_saved(&mut self, deserialized: SerializeableTimerParts) {
        let new_timer = Timer::from_serializeable(deserialized);

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
                if let TimerState::Alarming = self.timers[i].state {
                    self.alarm_counter.decrease_counter()
                }
                self.timers.remove(i);

                self.selected_timer = {
                    let s = self.timers.len();

                    if s == 0 {
                        None
                    } else if s == i {
                        Some(s - 1)
                    } else {
                        Some(i)
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
        if let Some(i) = self.selected_timer {
            match self.timers[i].state {
                TimerState::Stopped => {
                    self.timers[i].state = TimerState::Running;
                    self.timers[i].last_started = Some(Instant::now());
                }
                TimerState::Running => (),
                TimerState::Alarming => (),
            }
        }
    }

    fn pause_timer(&mut self) {
        if let Some(i) = self.selected_timer {
            match self.timers[i].state {
                TimerState::Running => {
                    self.timers[i].state = TimerState::Stopped;
                    self.timers[i].last_started = Some(Instant::now());
                    self.timers[i].time_left_at_last_tick =
                        self.timers[i].serializeable_parts.time_left;
                }
                TimerState::Alarming => {
                    // Change this! todo!()
                    // self.alarming_timers -= 1;
                    self.alarm_counter.decrease_counter();
                    self.timers[i].state = TimerState::Stopped;
                    self.timers[i].last_started = None;
                    self.timers[i].serializeable_parts.time_left =
                        self.timers[i].serializeable_parts.length;
                    self.timers[i].time_left_at_last_tick =
                        self.timers[i].serializeable_parts.length;
                }
                TimerState::Stopped => (),
            }
        }
    }

    fn toggle_timer(&mut self) {
        if let Some(i) = self.selected_timer {
            match self.timers[i].state {
                TimerState::Running => self.pause_timer(),
                TimerState::Stopped => self.start_timer(),
                // Don't need the alarming branch for pause because nothing
                // actually calls it?
                TimerState::Alarming => (),
            }
        }
    }

    fn reset_timer(&mut self) {
        if let Some(i) = self.selected_timer {
            match self.timers[i].state {
                TimerState::Running => (),
                TimerState::Stopped => {
                    // self.timers[i].state = TimerState::Stopped;
                    self.timers[i].serializeable_parts.time_left =
                        self.timers[i].serializeable_parts.length;
                    self.timers[i].time_left_at_last_tick =
                        self.timers[i].serializeable_parts.length;
                    self.timers[i].last_started = None;
                }
                // Change app alarming state here!
                TimerState::Alarming => {
                    // self.alarming_timers -= 1;
                    self.alarm_counter.decrease_counter();
                    self.timers[i].state = TimerState::Stopped;
                    self.timers[i].last_started = None;
                    self.timers[i].serializeable_parts.time_left =
                        self.timers[i].serializeable_parts.length;
                    self.timers[i].time_left_at_last_tick =
                        self.timers[i].serializeable_parts.length;
                }
            }
        }
    }

    fn edit_timer(&mut self) {
        if let Some(i) = self.selected_timer {
            self.screen = AppScreen::Editing(EditField::Description);
            self.edit_values = EditValues::new(
                self.timers[i].serializeable_parts.description.clone(),
                self.timers[i].serializeable_parts.length,
            )
        }
    }

    fn replace_timer(&mut self) {
        let new_timer = Timer::new(
            self.edit_values.descript.clone().into_lines().join(""),
            self.edit_values.to_duration(),
        );

        if let Some(i) = self.selected_timer {
            if let TimerState::Alarming = self.timers[i].state {
                self.alarm_counter.decrease_counter()
            }
            self.timers[i] = new_timer;
        }
    }

    fn add_new_timer(&mut self) {
        self.add_default_timer();
        self.selected_timer = Some(self.timers.len() - 1);

        self.screen = AppScreen::Editing(EditField::Description);
        self.edit_values = EditValues::default();
    }

    pub fn handle_events(&mut self) -> Result<(), Box<dyn Error>> {
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    match &self.screen {
                        AppScreen::Main => match key.code {
                            KeyCode::Char('q') => self.should_quit = true,
                            // KeyCode::Char('Q') => self.should_quit = true,
                            KeyCode::Char('k') => self.decrement_selection(),
                            KeyCode::Char('j') => self.increment_selection(),
                            KeyCode::Char('a') => self.add_new_timer(),
                            KeyCode::Char('d') => self.delete_timer(),
                            KeyCode::Char('p') => self.toggle_timer(),
                            KeyCode::Char('r') => self.reset_timer(),
                            KeyCode::Char('e') => self.edit_timer(),
                            _ => (),
                        },

                        AppScreen::Editing(edit_field) => match edit_field {
                            EditField::Description => match key.code {
                                KeyCode::Tab => self.screen = AppScreen::Editing(EditField::Hours1),
                                KeyCode::BackTab => {
                                    self.screen = AppScreen::Editing(EditField::Seconds2)
                                }
                                KeyCode::Enter => {
                                    self.replace_timer();
                                    self.screen = AppScreen::Main;
                                }
                                _ => {
                                    self.edit_values.descript.input(key);
                                } // _ => todo!(),
                            },
                            _ => match key.code {
                                KeyCode::Tab
                                | KeyCode::Right
                                | KeyCode::Char('l')
                                | KeyCode::Char('H') => {
                                    self.screen = AppScreen::Editing(EDITFIELD_NEXT[edit_field])
                                }
                                KeyCode::BackTab
                                | KeyCode::Left
                                | KeyCode::Char('h')
                                | KeyCode::Char('L') => {
                                    self.screen = AppScreen::Editing(EDITFIELD_PREVIOUS[edit_field])
                                }
                                // KeyCode::Char('h') => {
                                //     self.screen = AppScreen::Editing(EDITFIELD_HOUR[edit_field])
                                // }
                                // KeyCode::Char('m') => {
                                //     self.screen = AppScreen::Editing(EDITFIELD_MINUTE[edit_field])
                                // }
                                // KeyCode::Char('s') => {
                                //     self.screen = AppScreen::Editing(EDITFIELD_SECOND[edit_field])
                                // }
                                KeyCode::Char('k') => {
                                    self.edit_values.get_field(edit_field).inc_value()
                                }
                                KeyCode::Char('j') => {
                                    self.edit_values.get_field(edit_field).dec_value()
                                }
                                KeyCode::Enter => {
                                    self.replace_timer();
                                    self.screen = AppScreen::Main;
                                }
                                KeyCode::Char(x) => {
                                    if let Some(x) = x.to_digit(10) {
                                        self.edit_values.get_field(edit_field).set_value(x.into());
                                        self.screen =
                                            AppScreen::Editing(EDITFIELD_NEXT[edit_field]);
                                    }
                                }
                                _ => (),
                            },
                        },
                        AppScreen::Navigating => todo!(),
                        AppScreen::Error(_) => match key.code {
                            KeyCode::Char('q') => self.should_quit = true,
                            KeyCode::Enter => self.screen = AppScreen::Main,
                            _ => (),
                        },
                    }
                }
            }
        }
        Ok(())
    }

    pub fn dump_json(&self) -> String {
        let mut json = String::new();

        let mut timers_slice = Vec::with_capacity(self.timers.len());

        for timer in &self.timers {
            // write!(
            //     &mut json,
            //     "{},\n",
            //     serde_json::to_string_pretty(&timer.serializeable_parts).unwrap()
            // )
            // .unwrap();
            timers_slice.push(&timer.serializeable_parts);
        }

        write!(
            &mut json,
            "{}",
            serde_json::to_string_pretty(&timers_slice).unwrap()
        )
        .unwrap();

        json
    }

    pub fn read_from_json(&mut self, json: &str) -> Result<(), serde_json::Error> {
        // let timers: Vec<SerializeableTimerParts> = serde_json::from_str(&json).unwrap();
        let timers: Vec<SerializeableTimerParts> = serde_json::from_str(json)?;

        for timer in timers {
            self.add_timer_from_saved(timer);
        }

        Ok(())
    }
}
