use std::{
    error::Error,
    fs::{create_dir_all, File},
    io::{self, BufReader, Read, Write},
    path::PathBuf,
    thread::sleep,
    time::Duration,
};

use app::App;
use clap::Parser;
use config::Config;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use dirs::config_dir;
use ratatui::{prelude::CrosstermBackend, Terminal};
use ui::run_app;

mod app;
mod ui;

// const DEFAULT_CONFIG_FILE: &str = "/home/jeanpierre/.config/timers_tui/config.toml";
// const DEFAULT_SAVE_FILE: &str = "/home/jeanpierre/.config/timers_tui/saved_timers.json";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct CliOpts {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(short, long, value_name = "FILE")]
    save_file: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = CliOpts::parse();

    // let config_filename = match args.config {
    //     Some(filename) => filename,
    //     None => PathBuf::from(DEFAULT_CONFIG_FILE),
    // };

    let config_filename = match args.config {
        Some(filename) => Some(filename),
        None => match config_dir() {
            Some(mut config_path) => {
                config_path.push("timers_tui");
                config_path.push("config.toml");
                Some(config_path)
            }
            None => None,
        },
    };

    // let default_map_generator = || -> HashMap<String, String> {
    //     let mut map = HashMap::new();
    //
    //     println!("Had to use the default generator!");
    //
    //     map.insert(String::from("save_file"), String::from(DEFAULT_SAVE_FILE));
    //
    //     map
    // };
    let config_options = {
        let settings = Config::builder();

        // let settings = match config_dir() {
        //     Some(filename) => {
        //         let mut save_filename = filename;
        //         save_filename.push("timers_tui");
        //         save_filename.push("saved_timers.json");
        //         // println!("{}", save_filename.to_str().unwrap());
        //         settings.set_default("save_file", save_filename.to_str().unwrap())?
        //     }
        //     None => settings,
        // };

        match config_filename {
            Some(filename) => {
                let settings = settings.add_source(config::File::from(filename));
                match settings.build() {
                    Ok(options) => Some(options),
                    Err(_) => None,
                }
            }
            None => match settings.build() {
                Ok(options) => Some(options),
                Err(_) => None,
            },
        }
    };

    // println!("{:?}", config_options);

    let mut app = App::new();

    let timers_filename = match args.save_file {
        Some(filename) => Some(filename),
        None => match config_options {
            Some(options) => match options.get(&String::from("save_file")) {
                Ok(filename) => Some(filename),
                Err(_) => None,
            },
            None => match config_dir() {
                Some(mut filename) => {
                    filename.push("timers_tui");
                    filename.push("saved_timers.json");
                    Some(filename)
                }
                None => None,
            },
        },
    };

    load_timers(&mut app, &timers_filename);

    enable_raw_mode()?;
    let mut stderr = io::stderr();

    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    match app.successful_init {
        true => {
            let _ = run_app(&mut app, &mut terminal);
        }
        false => {
            println!("WARNING: Could not initialize sound playback. You will not hear any sound when an alarm goes off.");

            sleep(Duration::from_millis(5000));

            let _ = run_app(&mut app, &mut terminal);
        }
    }

    // run_app(&mut app, &mut terminal)?;

    // STUFF HERE
    // sleep(Duration::from_secs(5));

    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    save_timers(&app, &timers_filename);

    // println!("{}", app.dump_json());

    Ok(())
}

fn load_timers(app: &mut App, input_filename: &Option<PathBuf>) {
    // let input_filename = String::from("saved_timers.json");

    match input_filename {
        Some(input_filename) => {
            let input_file = File::open(input_filename);

            match input_file {
                Ok(file) => {
                    let mut reader = BufReader::new(file);

                    let mut contents = String::new();

                    reader.read_to_string(&mut contents).unwrap();

                    match app.read_from_json(&contents) {
                        Ok(_) => (),
                        Err(_) => println!("Could not parse the saved timers json file correctly"),
                    }
                }
                Err(_) => println!("Could not open the saved timers json file"),
            }
        }
        None => println!("Could not find a command line argument, configuration option, or default value specifying which file to load timers from"), 
    }
}

fn write_timers_file(app: &App, output_filename: &PathBuf) {
    let output_file = File::create(output_filename);

    match output_file {
        Ok(mut file) => match file.write_all(app.dump_json().as_bytes()) {
            Ok(_) => (),
            Err(_) => eprintln!("Could not save timers to json file"),
        },
        Err(_) => eprintln!("Could not open json file to save timers to"),
    }
}

fn save_timers(app: &App, output_filename: &Option<PathBuf>) {
    // let output_filename = String::from("saved_timers.json");
    match output_filename {
        Some(output_filename) => {
            if !output_filename.exists() {
                match output_filename.parent() {
                    Some(filepath) => match create_dir_all(filepath) {
                        Ok(_) => write_timers_file(app, &output_filename),
                        Err(_) => eprintln!("Could not create directory to save timers file to"),
                    },
                    None => eprintln!("Cannot save timers to '/'"),
                }
            } else {
                write_timers_file(app, &output_filename);
            }
        }
        None => println!("Could not find a command line argument, configuration option, or default value specifying which file to save timers to"),
    }
}
