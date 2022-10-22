use std::path::PathBuf;
use std::{io, net};

use clap::Parser;

/// PLONK debugger CLI
#[derive(Parser, Debug, Default)]
#[clap(author, version, about)]
pub struct Args {
    /// CDF file path
    #[clap(value_parser)]
    path: Option<PathBuf>,

    /// DAP backend to attach
    #[clap(long)]
    attach: Option<net::SocketAddr>,
}

impl Args {
    /// Resolve a command
    pub fn resolve(self) -> io::Result<ParsedArgs> {
        let Args { path, attach } = self;

        let path = match path {
            Some(p) => Some(p.canonicalize()?),
            None => None,
        };

        Ok(ParsedArgs { path, attach })
    }
}

/// Parsed arguments for the CLI
pub struct ParsedArgs {
    /// Path to the CDF file
    pub path: Option<PathBuf>,
    /// Socket to attach. Will bind to localhost if absent
    pub attach: Option<net::SocketAddr>,
}

#[test]
fn parse_default_args_wont_panic() {
    Args::default()
        .resolve()
        .expect("failed to parse default args");
}
