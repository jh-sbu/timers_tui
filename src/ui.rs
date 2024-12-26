use std::{collections::HashSet, error::Error};

use lazy_static::lazy_static;
use ratatui::{
    prelude::{Backend, Constraint, Direction, Layout},
    style::{Modifier, Style, Stylize},
    widgets::{Block, BorderType, Borders, HighlightSpacing, Paragraph, Row, Table, TableState},
    Terminal,
};

use crate::app::{App, AppScreen, EditField};

// const HOURS_SET: HashSet<EditField> =
// const MINUTES_SET: HashSet<EditField> = HashSet::from([EditField::Minutes1, EditField::Minutes2]);
// const SECONDS_SET: HashSet<EditField> = HashSet::from([EditField::Seconds1, EditField::Seconds2]);

lazy_static! {
    static ref HOURS_SET: HashSet<EditField> =
        HashSet::from([EditField::Hours1, EditField::Hours2, EditField::Hours3]);
    static ref MINUTES_SET: HashSet<EditField> =
        HashSet::from([EditField::Minutes1, EditField::Minutes2]);
    static ref SECONDS_SET: HashSet<EditField> =
        HashSet::from([EditField::Seconds1, EditField::Seconds2]);
}

fn build_block(title: String, highlighted: bool) -> Block<'static> {
    if highlighted {
        Block::default().borders(Borders::ALL).title(title).blue()
    } else {
        Block::default().borders(Borders::ALL).title(title)
    }
}

fn build_paragraph(value: String, highlighted: bool) -> Paragraph<'static> {
    if highlighted {
        Paragraph::new(value).style(Style::default()).red()
    } else {
        Paragraph::new(value).style(Style::default())
    }
}

pub fn run_app<B: Backend>(
    app: &mut App,
    terminal: &mut Terminal<B>,
) -> Result<(), Box<dyn Error>> {
    let mut state = TableState::default().with_selected(app.selected_timer);

    while !app.should_quit {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(5),
                    Constraint::Percentage(85),
                    Constraint::Percentage(10),
                ])
                .split(f.area());

            let title_block = Block::default().title("Timers").borders(Borders::NONE);

            f.render_widget(title_block, chunks[0]);

            let mut timer_block_rows = Vec::<Row>::new();

            for (i, timer) in app.timers.iter().enumerate() {
                let mut timer_row = Vec::new();

                timer_row.push(format!("{:4}", i));
                timer_row.push(timer.clone_description());

                timer_row.push(format!(
                    "{}:{:02}:{:02}",
                    timer.get_length().as_secs() / 3600,
                    timer.get_length().as_secs() % 3600 / 60,
                    timer.get_length().as_secs() % 60,
                ));

                timer_row.push(format!(
                    "{}:{:02}:{:02}",
                    timer.get_time_left().as_secs() / 3600,
                    timer.get_time_left().as_secs() % 3600 / 60,
                    timer.get_time_left().as_secs() % 60,
                ));

                let state_strslice = match timer.state {
                    crate::app::TimerState::Stopped => "Stopped",
                    crate::app::TimerState::Running => "Running",
                    crate::app::TimerState::Alarming => "Alarming",
                };
                timer_row.push(state_strslice.to_string());
                // timer_row.push(timer_box.timer.running);
                //
                timer_block_rows.push(Row::new(timer_row));
            }

            state.select(app.selected_timer);

            // let mut state = TableState::default().with_selected(app.selected_timer);
            // let mut state = TableState::default().with_selected(Some(0));

            let widths = [
                    Constraint::Percentage(10),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(15),
            ];

            let timer_block_table = Table::new(timer_block_rows, widths)
                .header(Row::new(vec![
                    "Timer #",
                    "Timer Description",
                    "Timer Length",
                    "Time Left",
                    "Status",
                ]))
                .block(Block::default().title("Timers").borders(Borders::ALL))
                .row_highlight_style(
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .add_modifier(Modifier::UNDERLINED),
                )
                .highlight_symbol(">>")
                .highlight_spacing(HighlightSpacing::Always);

            //f.render_widget(timer_block_table, chunks[1]);

            f.render_stateful_widget(timer_block_table, chunks[1], &mut state);

            // let mut timer_list_items = Vec::<ListItem>::new();
            //
            // for timer_box in app.timers {
            //     timer_list_items.push(ListItem::new(timer_box.title.clone()));
            // }
            //
            // let timer_block_list = List::new(timer_list_items)
            //     .block(Block::default().title("Timers").borders(Borders::ALL));
            //
            // f.render_widget(timer_block_list, chunks[1]);

            let commands_block = Block::default()
                .title("Commands")
                .borders(Borders::ALL)
                .border_style(Style::default())
                .border_type(BorderType::Rounded);

            // let commands_paragraph = Paragraph::new("No help text available").block(commands_block);

            let commands_paragraph = match &app.screen {
                AppScreen::Main => Paragraph::new("(q) - Quit | (j) - Select Next Timer | (k) Select Previous Timer | (a) - Add Timer | (d) - Delete Timer | (p) - Toggle Timer | (r) - Reset Timer | (e) - Edit Timer").block(commands_block),
                AppScreen::Editing(edit_field) => {
                    match edit_field {
                        EditField::Description => Paragraph::new("(Tab) - Switch Field | (Enter) - Accept").block(commands_block),
                        _ => Paragraph::new("(Tab) - Switch Field | (Enter) - Accept | (j) - Decrement | (k) - Increment | (0-9) - Set Value").block(commands_block),
                    }
                },
                AppScreen::Navigating => todo!(),
                AppScreen::Error(_) => Paragraph::new("(q) - Quit").block(commands_block),
            };

            f.render_widget(commands_paragraph, chunks[2]);

            match &app.screen {
                AppScreen::Editing(edit_screen) => {
                    // let editing_block = Block::default().borders(Borders::ALL).title("New Timer").title
                    let editing_layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(40),
                            Constraint::Percentage(20),
                            Constraint::Percentage(40),
                        ])
                        .split(f.area());

                    let editing_layout = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(40),
                            Constraint::Percentage(20),
                            Constraint::Percentage(40),
                        ])
                        .split(editing_layout[1]);

                    let editing_block = Block::default().borders(Borders::ALL).title("New Timer");

                    f.render_widget(editing_block, editing_layout[1]);

                    let editing_layout = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(60),
                            Constraint::Percentage(20),
                            Constraint::Percentage(10),
                            Constraint::Percentage(10),
                        ])
                        .margin(1)
                        .split(editing_layout[1]);

                    app.edit_values.descript.set_block(build_block(
                        String::from("Description"),
                        *edit_screen == EditField::Description,
                    ));

                    let desc_input = &app.edit_values.descript;

                    let hours_block =
                        build_block(String::from("HHH"), HOURS_SET.contains(edit_screen));
                    let minutes_block =
                        build_block(String::from("MM"), MINUTES_SET.contains(edit_screen));
                    let seconds_block =
                        build_block(String::from("SS"), SECONDS_SET.contains(edit_screen));

                    f.render_widget(hours_block, editing_layout[1]);
                    f.render_widget(minutes_block, editing_layout[2]);
                    f.render_widget(seconds_block, editing_layout[3]);

                    let hours_layout = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(33),
                            Constraint::Percentage(33),
                            Constraint::Percentage(34),
                        ])
                        .margin(1)
                        .split(editing_layout[1]);

                    let minutes_layout = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .margin(1)
                        .split(editing_layout[2]);

                    let seconds_layout = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                        .margin(1)
                        .split(editing_layout[3]);

                    let h1 = build_paragraph(
                        app.edit_values.hours1.value_as_string(),
                        *edit_screen == EditField::Hours1,
                    );
                    let h2 = build_paragraph(
                        app.edit_values.hours2.value_as_string(),
                        *edit_screen == EditField::Hours2,
                    );
                    let h3 = build_paragraph(
                        app.edit_values.hours3.value_as_string(),
                        *edit_screen == EditField::Hours3,
                    );

                    f.render_widget(h1, hours_layout[0]);
                    f.render_widget(h2, hours_layout[1]);
                    f.render_widget(h3, hours_layout[2]);

                    let m1 = build_paragraph(
                        app.edit_values.minutes1.value_as_string(),
                        *edit_screen == EditField::Minutes1,
                    );
                    let m2 = build_paragraph(
                        app.edit_values.minutes2.value_as_string(),
                        *edit_screen == EditField::Minutes2,
                    );

                    f.render_widget(m1, minutes_layout[0]);
                    f.render_widget(m2, minutes_layout[1]);

                    let s1 = build_paragraph(
                        app.edit_values.seconds1.value_as_string(),
                        *edit_screen == EditField::Seconds1,
                    );
                    let s2 = build_paragraph(
                        app.edit_values.seconds2.value_as_string(),
                        *edit_screen == EditField::Seconds2,
                    );

                    f.render_widget(s1, seconds_layout[0]);
                    f.render_widget(s2, seconds_layout[1]);
                    // let hours_input = Paragraph::new(app.edit_values.hours.to_string()).block(
                    //     build_block(String::from("HH"), EditField::Hours, edit_screen),
                    // );
                    //
                    // let minutes_input = Paragraph::new(app.edit_values.minutes.to_string()).block(
                    //     build_block(String::from("MM"), EditField::Minutes, edit_screen),
                    // );
                    //
                    // let seconds_input = Paragraph::new(app.edit_values.seconds.to_string()).block(
                    //     build_block(String::from("SS"), EditField::Seconds, edit_screen),
                    // );
                    //
                    // f.render_widget(not_implemented_desc_input, editing_layout[0]);
                    f.render_widget(desc_input, editing_layout[0]);
                    // f.render_widget(hours_input, editing_layout[1]);
                    // f.render_widget(minutes_input, editing_layout[2]);
                    // f.render_widget(seconds_input, editing_layout[3]);
                }
                AppScreen::Error(error_type) => {
                    let error_layout = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([
                            Constraint::Percentage(40),
                            Constraint::Percentage(20),
                            Constraint::Percentage(40),
                        ])
                        .split(f.area());

                    let error_layout = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([
                            Constraint::Percentage(40),
                            Constraint::Percentage(20),
                            Constraint::Percentage(40),
                        ])
                        .split(error_layout[1]);

                    let error_text = match error_type {
                        crate::app::ErrorType::SoundDevice => String::from("Could not open sound device. You will not hear any sound when the alarm goes off (q) Quit (Enter) Continue Anyway"),
                        crate::app::ErrorType::File => String::from("Could not open alarm sound file. Trying to use backup alarm sound (q) Quit (Enter) Continue Anyway"),
                    };

                    let error_paragraph = Paragraph::new(error_text)
                        .block(Block::default().borders(Borders::ALL))
                        .style(Style::default());

                    f.render_widget(error_paragraph, error_layout[1]);
                }
                _ => (),
            }
        })?;

        app.handle_events()?;

        app.update_timers();
    }
    Ok(())
}
