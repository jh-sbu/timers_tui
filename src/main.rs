use std::{error::Error, io};

use app::App;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::{Backend, Constraint, CrosstermBackend, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, BorderType, Borders, HighlightSpacing, Row, Table, TableState},
    Terminal,
};

mod app;

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stderr = io::stderr();

    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();

    app.add_timer();
    app.add_timer();

    for _ in 1..=10 {
        app.add_timer();
    }

    // while !app.should_quit {
    //     run_app(&mut app, &mut terminal)?;
    //
    //     app.handle_events()?;
    // }

    run_app(&mut app, &mut terminal)?;

    // STUFF HERE
    // sleep(Duration::from_secs(5));

    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_app<B: Backend>(app: &mut App, terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
    while !app.should_quit {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(5),
                    Constraint::Percentage(85),
                    Constraint::Percentage(10),
                ])
                .split(f.size());

            let title_block = Block::default().title("Timers").borders(Borders::NONE);

            f.render_widget(title_block, chunks[0]);

            let mut timer_block_rows = Vec::<Row>::new();

            for (i, timer) in app.timers.iter().enumerate() {
                let mut timer_row = Vec::new();

                timer_row.push(format!("{:4}", i));
                timer_row.push(timer.title.clone());
                timer_row.push(format!(
                    "{}.{:04}",
                    timer.time_left.as_secs(),
                    timer.time_left.as_millis()
                ));
                // timer_row.push(timer_box.timer.time_left);
                timer_row.push(format!(
                    "{}.{:04}",
                    timer.length.as_secs(),
                    timer.length.as_millis()
                ));
                // timer_row.push(timer_box.timer.length);
                timer_row.push(format!("{}", timer.running));
                // timer_row.push(timer_box.timer.running);
                //
                timer_block_rows.push(Row::new(timer_row));
            }

            let mut state = TableState::default().with_selected(app.selected_timer);
            // let mut state = TableState::default().with_selected(Some(0));

            let timer_block_table = Table::new(timer_block_rows)
                .header(Row::new(vec![
                    "Timer #",
                    "Timer Description",
                    "Duration Left",
                    "Total",
                    "Running",
                ]))
                .widths(&[
                    Constraint::Percentage(10),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(15),
                ])
                .block(Block::default().title("Timers").borders(Borders::ALL))
                .highlight_style(Style::default().add_modifier(Modifier::BOLD))
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

            f.render_widget(commands_block, chunks[2]);
        })?;

        app.handle_events()?;

        app.update_timers();
    }
    Ok(())
}

// fn timers_test() {
//     let mut timers = Vec::new();
//
//     for i in 1..=5 {
//         timers.push(Timer::new(Duration::from_secs(5 * i)));
//     }
//
//     while !&timers.is_empty() {
//         timers.retain(|i| {
//             if let Some(t) = i.time_left() {
//                 println!("{}", format!("{}.{:03}", t.as_secs(), t.subsec_millis()));
//                 true
//             } else {
//                 false
//             }
//         });
//
//         sleep(Duration::from_millis(17));
//     }
// }
