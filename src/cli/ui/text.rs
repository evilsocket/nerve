use std::{io::Write, time::Duration};

use colored::Colorize;

use crate::{
    agent::{
        events::{EventType, Receiver},
        namespaces::ActionOutput,
        ToolCall,
    },
    cli::Args,
    APP_NAME, APP_VERSION,
};

fn on_action_about_to_execute(tool_call: ToolCall) {
    let mut view = String::new();

    view.push_str("🧠 ");
    view.push_str(&tool_call.tool_name.bold().to_string());
    view.push('(');
    if let Some(payload) = &tool_call.argument {
        view.push_str(&payload.dimmed().to_string());
    }
    if let Some(attributes) = &tool_call.named_arguments {
        view.push_str(", ");
        view.push_str(
            &attributes
                .iter()
                .map(|(k, v)| format!("{}={}", k, v).dimmed().to_string())
                .collect::<Vec<String>>()
                .join(", "),
        );
    }
    view.push(')');

    log::info!("{} ...", view);
}

fn on_action_executed(
    judge_mode: bool,
    error: Option<String>,
    tool_call: ToolCall,
    result: Option<ActionOutput>,
    elapsed: Duration,
    complete_task: bool,
) {
    if judge_mode {
        if complete_task {
            if let Some(err) = error {
                println!("ERROR: {}", err);
            } else if let Some(res) = result {
                println!("{}", res);
            }
        }
        return;
    }

    let mut view = String::new();

    view.push_str("🛠️  ");
    view.push_str(&tool_call.tool_name);
    view.push_str(&format!(
        "({})",
        if tool_call.argument.is_some() {
            "..."
        } else {
            ""
        }
    ));

    if let Some(err) = error {
        log::error!("{}: {}", view, err);
    } else if let Some(res) = result {
        log::info!(
            "{}",
            format!(
                "{} -> {} bytes in {:?}",
                view,
                res.to_string().as_bytes().len(),
                elapsed
            )
            .dimmed()
        );
    } else {
        log::info!(
            "{}",
            format!("{} {} in {:?}", view, "no output", elapsed).dimmed()
        );
    }
}

pub async fn consume_events(mut events_rx: Receiver, args: Args, is_workflow: bool) {
    while let Some(event) = events_rx.recv().await {
        if let Some(record_to) = &args.record_to {
            let data = serde_json::to_string(&event).unwrap() + "\n";
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(record_to)
                .unwrap();

            file.write_all(data.as_bytes()).unwrap();
        }

        match event.event {
            EventType::WorkflowStarted(workflow) => {
                println!(
                    "{} v{} 🧠 | executing workflow {}\n",
                    APP_NAME,
                    APP_VERSION,
                    workflow.name.green().bold(),
                );
            }
            EventType::WorkflowCompleted(workflow) => {
                log::info!("workflow {} completed", workflow.name.green().bold());
                if let Some(report) = workflow.report {
                    println!("\n{}", report);
                }
            }
            EventType::TaskStarted(_task) => {}
            EventType::Sleeping(seconds) => {
                log::info!("💤 sleeping for {} seconds ...", seconds);
            }
            EventType::MetricsUpdate(metrics) => {
                log::info!("📊 {}", metrics.to_string().dimmed());
            }
            EventType::StateUpdate(_state) => {}
            EventType::Thinking(thinking) => {
                log::info!("🧠 thinking: {}", thinking.italic());
            }
            EventType::EmptyResponse => {
                log::warn!("🧠 {}", "...".dimmed());
            }
            EventType::TextResponse(response) => {
                log::info!("🧠 {}", response.trim().italic());
            }
            EventType::InvalidAction { tool_call, error } => {
                log::warn!("invalid action {} : {:?}", &tool_call.tool_name, error);
            }
            EventType::ActionTimeout { tool_call, elapsed } => {
                log::warn!(
                    "action '{}' timed out after {:?}",
                    tool_call.tool_name,
                    elapsed
                );
            }
            EventType::ActionExecuting { tool_call } => {
                on_action_about_to_execute(tool_call);
            }
            EventType::ActionExecuted {
                tool_call,
                error,
                result,
                elapsed,
                complete_task,
            } => {
                on_action_executed(
                    args.judge_mode,
                    error,
                    tool_call,
                    result,
                    elapsed,
                    complete_task,
                );
            }
            EventType::TaskComplete { impossible, reason } => {
                if !is_workflow {
                    if impossible {
                        if let Some(reason) = reason {
                            log::error!("{}: '{}'", "task is impossible".bold().red(), reason);
                        } else {
                            log::error!("{}", "task is impossible".bold().red());
                        }
                    } else if let Some(reason) = reason {
                        log::info!("{}: '{}'", "task complete".bold().green(), reason);
                    } else {
                        log::info!("{}", "task complete".bold().green());
                    }
                }
            }
            EventType::StorageUpdate {
                storage_name,
                storage_type: _,
                key,
                prev,
                new,
            } => {
                if !is_workflow {
                    if prev.is_none() && new.is_none() {
                        log::info!("storage.{} cleared", storage_name.yellow().bold());
                    } else if prev.is_none() && new.is_some() {
                        log::info!(
                            "storage.{}.{} > {}",
                            storage_name.yellow().bold(),
                            key,
                            new.unwrap().green()
                        );
                    } else if prev.is_some() && new.is_none() {
                        log::info!("{}.{} removed", storage_name.yellow().bold(), key);
                    } else if new.is_some() {
                        log::info!(
                            "{}.{} > {}",
                            storage_name.yellow().bold(),
                            key,
                            new.unwrap().green()
                        );
                    } else {
                        log::info!(
                            "{}.{} prev={:?} new={:?}",
                            storage_name.yellow().bold(),
                            key,
                            prev,
                            new
                        );
                    }
                }
            }
        }
    }
}
