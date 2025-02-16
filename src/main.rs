#![allow(dead_code)]
#![allow(clippy::module_inception)]

#[macro_use]
extern crate anyhow;

mod agent;
mod api;
mod cli;

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use agent::{
    events::{Event, EventType},
    task::variables::{define_variable, interpolate_variables},
    workflow::Workflow,
};
use anyhow::Result;
use cli::{setup, ui, Args};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

const APP_NAME: &str = env!("CARGO_BIN_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ControlState {
    Paused,
    Running,
    Stopped,
}

#[derive(Debug, Clone)]
pub struct Control {
    state: Arc<Mutex<ControlState>>,
}

impl Control {
    fn new(initial_state: ControlState) -> Self {
        Self {
            state: Arc::new(Mutex::new(initial_state)),
        }
    }

    async fn get_state(&self) -> ControlState {
        self.state.lock().await.clone()
    }

    async fn wait_if_paused(&self) {
        loop {
            let state = self.state.lock().await;
            if *state == ControlState::Paused {
                log::info!("waiting for control state to change from paused");
                std::thread::sleep(std::time::Duration::from_millis(1000));
            } else {
                log::debug!("control state is no longer paused");
                break;
            }
        }
    }

    async fn set_state(&self, new_state: ControlState) {
        let mut state = self.state.lock().await;
        *state = new_state.clone();
        log::info!("control state changed to {:?}", &new_state);
    }
}

async fn run_task(
    args: Args,
    for_workflow: bool,
    tx: agent::events::Sender,
    remote: Control,
) -> Result<HashMap<String, String>> {
    // single task
    let (mut agent, tasklet) =
        setup::setup_agent_for_task(&args, for_workflow, tx, remote.clone()).await?;

    // wait before starting the task
    remote.wait_if_paused().await;

    // signal the task start
    agent.on_event_type(EventType::TaskStarted(tasklet)).await?;

    // keep going until the task is complete or a fatal error is reached
    while !agent.is_done().await {
        // wait before every agent.step (which will wait internally in other sub phases)
        remote.wait_if_paused().await;

        // next step
        if let Err(error) = agent.step().await {
            log::error!("{}", error.to_string());
            return Err(error);
        }

        if let Some(sleep_seconds) = args.sleep {
            // signal the agent is sleeping
            agent
                .on_event_type(EventType::Sleeping(sleep_seconds))
                .await?;
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

async fn run_workflow(
    args: Args,
    workflow: &str,
    tx: agent::events::Sender,
    remote: Control,
) -> Result<()> {
    let mut workflow = Workflow::from_path(workflow)?;

    tx.send(Event::new(EventType::WorkflowStarted(workflow.clone())))?;

    for (task_name, task) in &workflow.tasks {
        // create the task specific arguments
        let task_args = get_workflow_task_args(&args, task_name, &workflow, task.generator.clone());
        // run the task as part of the workflow
        let variables = run_task(task_args, true, tx.clone(), remote.clone()).await?;
        // define variables for the next task
        for (key, value) in variables {
            define_variable(&key, &value);
        }
        println!();
    }

    if let Some(report) = workflow.report {
        workflow.report = Some(interpolate_variables(&report).await?);
    }

    tx.send(Event::new(EventType::WorkflowCompleted(workflow.clone())))?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = setup::setup_arguments().await?;

    // create main communication channel
    let (tx, _) = agent::events::create_channel();

    // spawn the terminal UI events consumer
    ui::text::start(tx.subscribe(), args.clone()).await?;

    let (remote, web_ui_handle) = if args.web_ui {
        // start in a paused state
        let remote = Control::new(ControlState::Paused);

        // start the webui
        let handle =
            ui::web::start(tx.subscribe(), tx.clone(), remote.clone(), args.clone()).await?;

        (remote, Some(handle))
    } else {
        (Control::new(ControlState::Running), None)
    };

    if let Some(workflow) = &args.workflow {
        // workflow
        run_workflow(args.clone(), workflow, tx, remote.clone()).await?;
    } else {
        // single task
        run_task(args, false, tx, remote.clone()).await?;
    }

    if let Some(handle) = web_ui_handle {
        handle.await?;
    }

    Ok(())
}
