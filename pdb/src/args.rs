use std::path::PathBuf;
use std::{io, net};

use clap::Parser;

/// PLONK debugger CLI
#[derive(Parser, Debug)]
#[clap(author, version, about)]
pub struct Args {
    /// CDF file path
    #[clap(value_parser)]
    path: Option<PathBuf>,

    /// Bind address for the DAP backend
    #[clap(long, default_value = "127.0.0.1")]
    ip: net::IpAddr,

    /// Bind port for the DAP backend
    #[clap(long, default_value = "0")]
    port: u16,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            path: Default::default(),
            ip: net::Ipv4Addr::LOCALHOST.into(),
            port: 0,
        }
    }
}

impl Args {
    /// Resolve a command
    pub fn resolve(self) -> io::Result<ParsedArgs> {
        let Args { path, ip, port } = self;

        let path = match path {
            Some(p) => Some(p.canonicalize()?),
            None => None,
        };

        let socket = net::SocketAddr::new(ip, port);

        Ok(ParsedArgs { path, socket })
    }
}

/// Parsed arguments for the CLI
pub struct ParsedArgs {
    /// Path to the CDF file
    pub path: Option<PathBuf>,
    /// Socket to bind DAP backend
    pub socket: net::SocketAddr,
}

#[test]
fn parse_default_args_wont_panic() {
    Args::default()
        .resolve()
        .expect("failed to parse default args");
}
