use std::{env, io};

use tracing_subscriber::filter::EnvFilter;

#[tokio::main]
async fn main() -> io::Result<()> {
    let filter = match env::var_os("RUST_LOG") {
        Some(_) => EnvFilter::try_from_default_env()
            .expect("Invalid `RUST_LOG` provided"),
        None => EnvFilter::new("info"),
    };

    tracing_subscriber::fmt::Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_env_filter(filter)
        .init();

    dusk_cdf::ZkDapBuilder::new("127.0.0.1:0")
        .build()
        .await?
        .listen()
        .await?;

    Ok(())
}
