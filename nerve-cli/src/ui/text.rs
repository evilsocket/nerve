use std::time::Duration;

use colored::Colorize;
use nerve_core::agent::Invocation;

use crate::{
    agent::events::{Event, Receiver},
    cli,
};

fn on_action_executed(
    error: Option<String>,
    invocation: Invocation,
    result: Option<String>,
    elapsed: Duration,
) {
    let mut view = String::new();

    view.push_str("🧠 ");
    view.push_str(&invocation.action.bold().to_string());
    view.push_str("(");
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
    view.push_str(")");

    if let Some(err) = error {
        log::error!("{}: {}", view, err);
    } else if let Some(res) = result {
        log::info!(
            "{} -> {} bytes in {:?}",
            view,
            res.as_bytes().len(),
            elapsed
        );
    } else {
        log::info!("{} {} in {:?}", view, "no output".dimmed(), elapsed);
    }
}

pub async fn consume_events(args: cli::Args, mut events_rx: Receiver) {
    while let Some(event) = events_rx.recv().await {
        match event {
            Event::MetricsUpdate(metrics) => {
                log::info!("{}", metrics.to_string().dimmed());
            }
            Event::StateUpdate(opts) => {
                if let Some(prompt_path) = &args.save_to {
                    let data = format!(
                        "[SYSTEM PROMPT]\n\n{}\n\n[PROMPT]\n\n{}\n\n[CHAT]\n\n{}",
                        &opts.system_prompt,
                        &opts.prompt,
                        opts.history
                            .iter()
                            .map(|m| m.to_string())
                            .collect::<Vec<String>>()
                            .join("\n")
                    );

                    if let Err(e) = std::fs::write(prompt_path, data) {
                        log::error!("error writing {}: {:?}", prompt_path, e);
                    }
                }
            }
            Event::EmptyResponse => {
                log::warn!("agent did not provide valid instructions: empty response");
            }
            Event::InvalidResponse(response) => {
                log::warn!(
                    "agent did not provide valid instructions: \n\n{}\n\n",
                    response.dimmed()
                );
            }
            Event::InvalidAction { invocation, error } => {
                log::warn!("invalid action {} : {:?}", &invocation.action, error);
            }
            Event::ActionTimeout {
                invocation,
                elapsed,
            } => {
                log::warn!(
                    "action '{}' timed out after {:?}",
                    invocation.action,
                    elapsed
                );
            }
            Event::ActionExecuted {
                invocation,
                error,
                result,
                elapsed,
            } => {
                on_action_executed(error, invocation, result, elapsed);
            }
            Event::TaskComplete { impossible, reason } => {
                if impossible {
                    log::error!(
                        "{}: '{}'",
                        "task is impossible".bold().red(),
                        if let Some(r) = &reason {
                            r
                        } else {
                            "no reason provided"
                        }
                    );
                } else {
                    log::info!(
                        "{}: '{}'",
                        "task complete".bold().green(),
                        if let Some(r) = &reason {
                            r
                        } else {
                            "no reason provided"
                        }
                    );
                }
            }
            Event::StorageUpdate {
                storage_name,
                storage_type: _,
                key,
                prev,
                new,
            } => {
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
