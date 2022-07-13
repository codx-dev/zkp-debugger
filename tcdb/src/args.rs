use std::path::PathBuf;
use std::{fs, io};

use clap::Parser;

/// PLONK debugger CLI
#[derive(Parser, Debug, Default)]
#[clap(author, version, about)]
pub struct Args {
    /// Configuration dir
    #[clap(short, long, value_parser)]
    config_dir: Option<PathBuf>,

    /// CDF file path
    #[clap(short, long, value_parser)]
    path: Option<PathBuf>,
}

impl Args {
    /// Resolve a command
    pub fn resolve(self) -> io::Result<ParsedArgs> {
        let Args { config_dir, path } = self;

        let config_dir = config_dir
            .or_else(|| dirs::config_dir().map(|d| d.join("pdb")))
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "failed to define config dir"))?;

        if config_dir.exists() {
            if !config_dir.is_dir() {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!(
                        "the provided config directory is not a directory: {}",
                        config_dir.display()
                    ),
                ));
            }
        } else {
            fs::create_dir_all(&config_dir)?;
        }

        let config_dir = config_dir.canonicalize()?;

        let mut commands_history = config_dir.clone();
        commands_history.push("commands_history");

        Ok(ParsedArgs {
            config_dir,
            commands_history,
            path,
        })
    }
}

/// Parsed arguments for the CLI
pub struct ParsedArgs {
    /// Directory to be used for the config files
    pub config_dir: PathBuf,
    /// File to store the commands history
    pub commands_history: PathBuf,
    /// Path to the CDF file
    pub path: Option<PathBuf>,
}

#[test]
fn parse_default_args_wont_panic() {
    Args::default()
        .resolve()
        .expect("failed to parse default args");
}
