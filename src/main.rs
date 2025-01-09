#![allow(dead_code)]
#![allow(clippy::module_inception)]

#[macro_use]
extern crate anyhow;

mod agent;
mod api;
mod cli;

use anyhow::Result;
use cli::{setup, ui};

const APP_NAME: &str = env!("CARGO_BIN_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    let args = setup::setup_arguments().await?;

    let (mut agent, events_rx) = setup::setup_agent(&args).await?;

    // spawn the events consumer
    tokio::spawn(ui::text::consume_events(args, events_rx));

    // keep going until the task is complete or a fatal error is reached
    while !agent.is_done().await {
        // next step
        if let Err(error) = agent.step().await {
            log::error!("{}", error.to_string());
            return Err(error);
        }
    }

    agent.on_end().await
}
