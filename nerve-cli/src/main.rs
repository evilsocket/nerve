#[macro_use]
extern crate anyhow;

use anyhow::Result;
use clap::Parser;
use nerve_core::agent;

mod cli;
mod setup;
mod ui;

const APP_NAME: &str = env!("CARGO_BIN_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: save/restore session
    let mut args = cli::Args::parse();

    // set generator url if env variable is set
    if let Ok(env_generator) = std::env::var("NERVE_GENERATOR") {
        args.generator = env_generator;
    }

    // set tasklet if env variable is set
    if let Ok(env_tasklet) = std::env::var("NERVE_TASKLET") {
        args.tasklet = Some(env_tasklet);
    }

    // TODO: handle max tokens

    if args.generate_doc {
        // generate action namespaces documentation and exit
        println!("{}", agent::serialization::available_actions());
        std::process::exit(0);
    }

    if std::env::var_os("RUST_LOG").is_none() {
        // set `RUST_LOG=debug` to see debug logs
        std::env::set_var("RUST_LOG", "info,openai_api_rust=warn");
    }

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_module_path(false)
        .format_target(false)
        .init();

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
