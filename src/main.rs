#![allow(dead_code)]
#![allow(clippy::module_inception)]

#[macro_use]
extern crate anyhow;

mod agent;
mod api;
mod cli;

use std::{collections::HashMap, fs::File, path::PathBuf};

use agent::{
    task::variables::{define_variable, get_variables, interpolate_variables},
    workflow::Workflow,
};
use anyhow::Result;
use cli::{setup, ui, Args};
use colored::Colorize;

const APP_NAME: &str = env!("CARGO_BIN_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

async fn run_task(args: Args, for_workflow: bool) -> Result<HashMap<String, String>> {
    // single task
    let (mut agent, events_rx) = setup::setup_agent_for_task(&args, for_workflow).await?;

    // spawn the events consumer
    tokio::spawn(ui::text::consume_events(
        events_rx,
        args.clone(),
        for_workflow,
    ));

    // keep going until the task is complete or a fatal error is reached
    while !agent.is_done().await {
        // next step
        if let Err(error) = agent.step().await {
            log::error!("{}", error.to_string());
            return Err(error);
        }
    }

    agent.on_end().await?;

    Ok(agent.get_variables().await)
}

fn get_workflow_task_args(
    args: Args,
    task_name: &str,
    workflow: &Workflow,
    generator: Option<String>,
) -> Args {
    let mut task_args = args.clone();
    task_args.tasklet = Some(
        PathBuf::from(&workflow.folder)
            .join(task_name)
            .with_extension("yml")
            .to_str()
            .unwrap()
            .to_string(),
    );
    if let Some(generator) = generator {
        task_args.generator = generator.clone();
    }

    task_args
}

async fn run_workflow(args: Args, workflow: &String) -> Result<()> {
    let workflow = Workflow::from_path(workflow)?;
    println!(
        "{} v{} 🧠 | executing workflow {}\n",
        APP_NAME,
        APP_VERSION,
        workflow.name.green().bold(),
    );

    for (task_name, task) in &workflow.tasks {
        // create the task specific arguments
        let task_args =
            get_workflow_task_args(args.clone(), &task_name, &workflow, task.generator.clone());
        // run the task as part of the workflow
        let variables = run_task(task_args, true).await?;
        // define variables for the next task
        for (key, value) in variables {
            define_variable(&key, &value);
        }
        println!();
    }

    log::info!("workflow {} completed\n", workflow.name.green().bold());

    if let Some(report) = workflow.report {
        println!("\n{}", interpolate_variables(&report).unwrap());
    }

    if let Some(output) = args.output {
        let mut file = File::create(&output)?;
        let variables = get_variables();
        serde_json::to_writer_pretty(&mut file, &variables)?;

        log::info!("output state saved to {}", output.green().bold());
    }

    return Ok(());
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = setup::setup_arguments().await?;

    if let Some(workflow) = &args.workflow {
        // workflow
        run_workflow(args.clone(), workflow).await?;
    } else {
        // single task
        run_task(args, false).await?;
    }

    Ok(())
}
