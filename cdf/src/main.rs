use std::{env, io, net};

use clap::Parser;
use tracing_subscriber::filter::EnvFilter;

#[derive(Parser, Debug, Default)]
#[clap(author, version, about)]
pub struct Args {
    #[clap(long)]
    bind: Option<net::SocketAddr>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let Args { bind } = Args::parse();

    let bind = bind.unwrap_or_else(|| {
        net::SocketAddr::new(net::Ipv4Addr::LOCALHOST.into(), 0)
    });

    let filter = env::var_os("RUST_LOG")
        .map(|_| EnvFilter::try_from_default_env())
        .transpose()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        .unwrap_or_else(|| EnvFilter::new("info"));

    tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();

    dusk_cdf::ZkDapBuilder::new(bind)
        .build()
        .await?
        .listen()
        .await?;

    Ok(())
}
