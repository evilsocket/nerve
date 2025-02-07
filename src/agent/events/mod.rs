use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

mod channel;

pub use channel::*;

use super::namespaces::ActionOutput;
use super::task::tasklet::Tasklet;
use super::workflow::Workflow;
use super::{
    generator::ChatOptions,
    state::{metrics::Metrics, storage::StorageType},
    Invocation,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct StateUpdate {
    pub chat: ChatOptions,
    pub globals: HashMap<String, String>,
    pub variables: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum EventType {
    WorkflowStarted(Workflow),
    WorkflowCompleted(Workflow),
    TaskStarted(Tasklet),
    MetricsUpdate(Metrics),
    StorageUpdate {
        storage_name: String,
        storage_type: StorageType,
        key: String,
        prev: Option<String>,
        new: Option<String>,
    },
    StateUpdate(StateUpdate),
    EmptyResponse,
    Thinking(String),
    Sleeping(usize),
    TextResponse(String),
    InvalidAction {
        invocation: Invocation,
        error: Option<String>,
    },
    ActionTimeout {
        invocation: Invocation,
        elapsed: std::time::Duration,
    },
    ActionExecuting {
        invocation: Invocation,
    },
    ActionExecuted {
        invocation: Invocation,
        error: Option<String>,
        result: Option<ActionOutput>,
        elapsed: std::time::Duration,
        complete_task: bool,
    },
    TaskComplete {
        impossible: bool,
        reason: Option<String>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub timestamp: u128,
    #[serde(flatten)]
    pub event: EventType,
}

impl Event {
    pub fn new(event: EventType) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            event,
        }
    }
}
