/*
use std::io;

use clap::Parser;
use crossterm::{cursor, execute, terminal};
use dusk_tcdb::*;
use rustyline::error::ReadlineError;
use rustyline::{Config as RustylineConfig, Editor};
*/

fn main() {
    /*
    let ParsedArgs {
        commands_history,
        path,
        ..
    } = Args::parse().resolve().expect("failed to resolve cli args");

    let mut app = App::load().expect("failed to load app");

    execute!(
        io::stdout(),
        terminal::EnterAlternateScreen,
        cursor::MoveTo(0, 0)
    )
    .expect("failed to load app screen");

    if let Some(p) = path {
        app.open_path(p).expect("failed to open provided CDF file");
    }

    let config = RustylineConfig::builder()
        .auto_add_history(true)
        .max_history_size(500)
        .build();

    let bell = format!("{} ", '\u{03c0}');

    let mut rl = Editor::<CommandParser>::with_config(config);

    rl.set_helper(Some(app.parser().clone()));
    rl.load_history(&commands_history).ok();

    loop {
        let readline = rl.readline(&bell);

        match readline {
            Ok(line) => match app.parse_and_execute(&line) {
                Ok(State::Continue) => (),
                Ok(State::ShouldQuit) => break,
                Err(e) => eprintln!("{}", e),
            },
            Err(ReadlineError::Interrupted) => {
                //eprintln!("CTRL-C");
            }
            Err(ReadlineError::Eof) => {
                eprintln!("CTRL-D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
            }
        }
    }

    execute!(io::stdout(), terminal::LeaveAlternateScreen).expect("failed to unload app screen");

    rl.save_history(&commands_history)
        .expect("failed to save commands history");
    */
}
