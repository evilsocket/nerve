#![allow(dead_code)]

#[macro_use]
extern crate anyhow;

mod agent;
mod api;
mod cli;

use anyhow::Result;
use clap::Parser;
use cli::{setup, ui};

const APP_NAME: &str = env!("CARGO_BIN_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: save/restore session
    let mut args = cli::Args::parse();

    // set generator url if env variable is set
    if let Ok(env_generator) = std::env::var("NERVE_GENERATOR") {
        args.generator = env_generator;
    } else {
        // set env variable for later use
        std::env::set_var("NERVE_GENERATOR", args.generator.clone());
    }

    // set judge url if env variable is set
    if let Ok(env_judge) = std::env::var("NERVE_JUDGE") {
        args.judge = env_judge;
    } else {
        // set env variable for later use
        std::env::set_var("NERVE_JUDGE", args.judge.clone());
    }

    // if we're running in judge mode, set the generator to the judge model
    if args.judge_mode {
        args.generator = args.judge.clone();
    }

    // set tasklet if env variable is set
    if let Ok(env_tasklet) = std::env::var("NERVE_TASKLET") {
        args.tasklet = Some(env_tasklet);
    }

    // TODO: handle max tokens

    if args.generate_doc {
        // generate action namespaces documentation and exit
        println!("{}", agent::serialization::Strategy::available_actions());
        std::process::exit(0);
    }

    if std::env::var_os("RUST_LOG").is_none() {
        // set `RUST_LOG=debug` to see debug logs
        std::env::set_var(
            "RUST_LOG",
            "info,openai_api_rust=warn,rustls=warn,ureq=warn",
        );
    }

    if args.judge_mode {
        // disable most logging
        std::env::set_var(
            "RUST_LOG",
            "error,openai_api_rust=error,rustls=error,ureq=error",
        );

        // read STDIN and preemptively set $STDIN
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_goes_into_input_above) => {}
            Err(_no_updates_is_fine) => {}
        }
        agent::task::variables::define_variable("STDIN", input.trim());
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
