use anyhow::{Context, Result};
use clap::Parser;
use config::CommandLine;
use log4rs::config::Deserializers;
use std::default::Default;

mod config;
mod database;
mod index;
mod library;
mod server;

static METRIC_DISALLOWED_PATH: &str = "disallowed_path_counter";

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CommandLine::parse();

    let config = cli.configuration()?;

    let path = config.log_config();
    log4rs::init_file(&path, Deserializers::default())
        .with_context(|| format!("log file {path}"))?;

    let path = config.indexing().path();
    let index =
        index::init_index(&path).with_context(|| format!("Index folder {}", path.display()))?;

    server::create(&config.listen(), index, config).await?;

    Ok(())
}
