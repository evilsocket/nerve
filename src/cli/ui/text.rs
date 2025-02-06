use std::{io::Write, time::Duration};

use colored::Colorize;

use crate::{
    agent::{
        events::{EventType, Receiver},
        namespaces::ActionOutput,
        Invocation,
    },
    cli::Args,
};

fn on_action_about_to_execute(invocation: Invocation) {
    let mut view = String::new();

    view.push_str("🧠 ");
    view.push_str(&invocation.action.bold().to_string());
    view.push('(');
    if let Some(payload) = &invocation.payload {
        view.push_str(&payload.dimmed().to_string());
    }
    if let Some(attributes) = &invocation.attributes {
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
    invocation: Invocation,
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
    view.push_str(&invocation.action);
    view.push_str(&format!(
        "({})",
        if invocation.payload.is_some() {
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
            EventType::TaskStarted(_task) => {}
            EventType::Sleeping(seconds) => {
                log::info!("💤 sleeping for {} seconds ...", seconds);
            }
            EventType::MetricsUpdate(metrics) => {
                log::debug!("{}", metrics.to_string());
            }
            EventType::StateUpdate(_state) => {}
            EventType::Thinking(thinking) => {
                log::info!("🧠 thinking: {}", thinking.dimmed());
            }
            EventType::EmptyResponse => {
                log::warn!("🧠 empty response");
            }
            EventType::InvalidResponse(response) => {
                log::info!("🧠 {}", response.trim().dimmed());
            }
            EventType::InvalidAction { invocation, error } => {
                log::warn!("invalid action {} : {:?}", &invocation.action, error);
            }
            EventType::ActionTimeout {
                invocation,
                elapsed,
            } => {
                log::warn!(
                    "action '{}' timed out after {:?}",
                    invocation.action,
                    elapsed
                );
            }
            EventType::ActionExecuting { invocation } => {
                on_action_about_to_execute(invocation);
            }
            EventType::ActionExecuted {
                invocation,
                error,
                result,
                elapsed,
                complete_task,
            } => {
                on_action_executed(
                    args.judge_mode,
                    error,
                    invocation,
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
                    } else {
                        if let Some(reason) = reason {
                            log::info!("{}: '{}'", "task complete".bold().green(), reason);
                        } else {
                            log::info!("{}", "task complete".bold().green());
                        }
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
