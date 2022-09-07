use std::io;
use std::path::PathBuf;

use clap::Parser;

/// PLONK debugger CLI
#[derive(Parser, Debug, Default)]
#[clap(author, version, about)]
pub struct Args {
    /// CDF file path
    #[clap(value_parser)]
    path: Option<PathBuf>,
}

impl Args {
    /// Resolve a command
    pub fn resolve(self) -> io::Result<ParsedArgs> {
        let Args { path } = self;

        let path = match path {
            Some(p) => Some(p.canonicalize()?),
            None => None,
        };

        Ok(ParsedArgs { path })
    }
}

/// Parsed arguments for the CLI
pub struct ParsedArgs {
    /// Path to the CDF file
    pub path: Option<PathBuf>,
}

#[test]
fn parse_default_args_wont_panic() {
    Args::default()
        .resolve()
        .expect("failed to parse default args");
}
