#[macro_use]
extern crate anyhow;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

mod agent;
mod cli;
mod setup;

const APP_NAME: &str = env!("CARGO_BIN_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: save/restore session

    let args = cli::Args::parse();

    if args.generate_doc {
        // generate action namespaces documentation and exit
        println!("{}", agent::serialization::available_actions());
        std::process::exit(0);
    }

    let mut agent = setup::setup_agent(&args).await?;

    // keep going until the task is complete or a fatal error is reached
    while !agent.is_done().await {
        // next step
        if let Err(error) = agent.step().await {
            println!("{}", error.to_string().bold().red());
            return Err(error);
        }
    }

    // report final metrics on exit
    if args.stats {
        println!("\n{}", agent.get_metrics().await);
    }

    Ok(())
}
