#![allow(dead_code)]
#![allow(clippy::module_inception)]

#[macro_use]
extern crate anyhow;

mod agent;
mod api;
mod cli;

use std::{collections::HashMap, path::PathBuf};

use agent::{
    events::EventType,
    task::variables::{define_variable, interpolate_variables},
    workflow::Workflow,
};
use anyhow::Result;
use cli::{setup, ui, Args};
use colored::Colorize;

const APP_NAME: &str = env!("CARGO_BIN_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

async fn run_task(args: Args, for_workflow: bool) -> Result<HashMap<String, String>> {
    // single task
    let (mut agent, tasklet, events_rx) = setup::setup_agent_for_task(&args, for_workflow).await?;

    // spawn the events consumer
    tokio::spawn(ui::text::consume_events(
        events_rx,
        args.clone(),
        for_workflow,
    ));

    // signal the task start
    agent.on_event_type(EventType::TaskStarted(tasklet))?;

    // keep going until the task is complete or a fatal error is reached
    while !agent.is_done().await {
        // next step
        if let Err(error) = agent.step().await {
            log::error!("{}", error.to_string());
            return Err(error);
        }

        if let Some(sleep_seconds) = args.sleep {
            // signal the agent is sleeping
            agent.on_event_type(EventType::Sleeping(sleep_seconds))?;
            // sleep for the given number of seconds
            tokio::time::sleep(std::time::Duration::from_secs(sleep_seconds as u64)).await;
        }
    }

    agent.on_end().await?;

    // return any defined variables
    Ok(agent.get_variables().await)
}

fn get_workflow_task_args(
    args: &Args,
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

async fn run_workflow(args: Args, workflow: &str) -> Result<()> {
    let workflow = Workflow::from_path(workflow)?;
    println!(
        "{} v{} 🧠 | executing workflow {}\n",
        APP_NAME,
        APP_VERSION,
        workflow.name.green().bold(),
    );

    for (task_name, task) in &workflow.tasks {
        // create the task specific arguments
        let task_args = get_workflow_task_args(&args, task_name, &workflow, task.generator.clone());
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

    Ok(())
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
