#![allow(dead_code)]
#![allow(clippy::module_inception)]

#[macro_use]
extern crate anyhow;

mod agent;
mod api;
mod cli;

use std::path::PathBuf;

use agent::{
    task::variables::{define_variable, interpolate_variables},
    workflow::Workflow,
};
use anyhow::Result;
use cli::{setup, ui};
use colored::Colorize;

const APP_NAME: &str = env!("CARGO_BIN_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> Result<()> {
    let args = setup::setup_arguments().await?;

    if let Some(workflow) = &args.workflow {
        let workflow = Workflow::from_path(workflow)?;
        println!(
            "{} v{} 🧠 | executing workflow {}\n",
            APP_NAME,
            APP_VERSION,
            workflow.name.green().bold(),
        );

        for (task_name, task) in workflow.tasks {
            let mut task_args = args.clone();

            task_args.tasklet = Some(
                PathBuf::from(&workflow.folder)
                    .join(task_name)
                    .with_extension("yml")
                    .to_str()
                    .unwrap()
                    .to_string(),
            );
            if let Some(generator) = &task.generator {
                task_args.generator = generator.clone();
            }

            let (mut agent, events_rx) = setup::setup_agent_for_task(&task_args, true).await?;

            // spawn the events consumer
            tokio::spawn(ui::text::consume_events(events_rx, None, false, true));
            // keep going until the task is complete or a fatal error is reached
            while !agent.is_done().await {
                // next step
                if let Err(error) = agent.step().await {
                    log::error!("{}", error.to_string());
                    return Err(error);
                }
            }

            agent.on_end().await?;

            // define variables for the next task
            for (key, value) in agent.get_variables().await {
                define_variable(&key, &value);
            }

            drop(agent);
            println!();
        }

        log::info!("workflow {} completed\n", workflow.name.green().bold());

        if let Some(report) = workflow.report {
            println!("\n{}", interpolate_variables(&report).unwrap());
        }

        return Ok(());
    } else {
        // single task
        let (mut agent, events_rx) = setup::setup_agent_for_task(&args, false).await?;

        // spawn the events consumer
        tokio::spawn(ui::text::consume_events(
            events_rx,
            args.save_to,
            args.judge_mode,
            false,
        ));

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
}
