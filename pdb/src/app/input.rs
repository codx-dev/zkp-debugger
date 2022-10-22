use std::path::PathBuf;
use std::{fs, io};

use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::commands::{Command, CommandParser};

use super::config::Config;

pub struct Input {
    rl: Editor<CommandParser>,
    bell: String,
    history: Option<PathBuf>,
    parser: CommandParser,
}

impl Input {
    pub fn help(&self) -> String {
        self.parser
            .instructions()
            .iter()
            .fold(String::new(), |mut s, i| {
                s.push_str(&format!("{} - {}\n", i.syntax(), i.help()));
                s
            })
    }
}

impl Iterator for Input {
    type Item = Command;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.rl.readline(&self.bell) {
                Ok(line) => match self.parser.parse(&line) {
                    Ok(Some(Command::Quit)) => return None,

                    Ok(Some(c)) => return Some(c),

                    Ok(None) => (),

                    Err(e) => eprintln!("error parsing command: {}", e),
                },
                Err(ReadlineError::Interrupted) => {
                    //eprintln!("CTRL-C");
                }
                Err(ReadlineError::Eof) => {
                    eprintln!("CTRL-D");
                    return None;
                }
                Err(err) => {
                    eprintln!("error reading command: {}", err);
                }
            }
        }
    }
}

impl TryFrom<&Config> for Input {
    type Error = io::Error;

    fn try_from(config: &Config) -> io::Result<Self> {
        let line = config.rustyline();
        let bell = format!("{} ", '\u{03c0}');
        let parser = CommandParser::default();

        let mut rl = Editor::<CommandParser>::with_config(line)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        rl.set_helper(Some(parser.clone()));

        let history = dirs::data_local_dir()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::Other,
                    "failed to fetch data local dir",
                )
            })
            .map(|p| p.join(env!("CARGO_PKG_NAME")))
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

        Ok(Self {
            rl,
            bell,
            history,
            parser,
        })
    }
}

impl Drop for Input {
    fn drop(&mut self) {
        if let Some(h) = &self.history {
            if let Err(e) = self.rl.save_history(h) {
                eprintln!("failed to save commands history: {}", e);
            }
        }
    }
}

#[test]
fn init_works() -> io::Result<()> {
    use toml_base_config::BaseConfig;

    let config = Config::load()?;

    Input::try_from(&config)?;

    Ok(())
}

#[test]
fn help_works() {
    use toml_base_config::BaseConfig;

    let config = Config::load().expect("failed to load config");
    let input = Input::try_from(&config).expect("failed to load input");

    input.help();
}
