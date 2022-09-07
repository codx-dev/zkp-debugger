use std::{fs, io};

use clap::Parser;
use crossterm::{cursor, execute, terminal};
use rustyline::error::ReadlineError;
use rustyline::Editor;

use dusk_pdb::prelude::*;

fn main() {
    let args = Args::parse().resolve().expect("failed to resolve cli args");

    execute!(
        io::stdout(),
        terminal::EnterAlternateScreen,
        cursor::MoveTo(0, 0)
    )
    .expect("failed to load app screen");

    let mut app = App::load(args).expect("failed to load app");
    let line_config = app.config().rustyline();

    let bell = format!("{} ", '\u{03c0}');
    let mut rl = Editor::<CommandParser>::with_config(line_config);

    rl.set_helper(Some(app.parser().clone()));

    let history = dirs::data_local_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "failed to fetch data local dir"))
        .map(|p| p.join(env!("CARGO_BIN_NAME")))
        .and_then(|p| {
            fs::create_dir_all(&p)?;
            Ok(p.join("history"))
        })
        .ok();

    if let Some(h) = &history {
        if !h.exists() {
            fs::OpenOptions::new().create_new(true).open(h).ok();
        }

        if h.exists() {
            if let Err(e) = rl.load_history(h) {
                eprintln!("failed to load commands history: {}", e);
            }
        }
    }

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

    if let Some(h) = &history {
        if let Err(e) = rl.save_history(h) {
            eprintln!("failed to save commands history: {}", e);
        }
    }
}
