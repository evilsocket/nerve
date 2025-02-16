use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

mod channel;

pub use channel::*;

use crate::ControlState;

use super::namespaces::ToolOutput;
use super::task::tasklet::Tasklet;
use super::workflow::Workflow;
use super::{
    generator::ChatOptions,
    state::{metrics::Metrics, storage::StorageType},
    ToolCall,
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
    // the control state has been updated
    ControlStateChanged(ControlState),
    // the workflow has been started
    WorkflowStarted(Workflow),
    // the workflow has been completed
    WorkflowCompleted(Workflow),
    // the task has been started
    TaskStarted(Tasklet),
    // new metrics are available
    MetricsUpdate(Metrics),
    // storage (memory or other parts of the prompt) state update
    StorageUpdate {
        // storage name and type
        storage_name: String,
        storage_type: StorageType,
        // key of the object that changed
        key: String,
        // previous value if pre-existed
        prev: Option<String>,
        // new value
        new: Option<String>,
    },
    // the state of the agent (system prompt, user prompt, conversation) has been updated
    StateUpdate(StateUpdate),
    // the agent provided an empty response
    EmptyResponse,
    // the agent is thinking (R1 and any reasoning model)
    Thinking(String),
    // the agent is sleeping for a given amount of seconds
    Sleeping(usize),
    // the agent provided a text response without tool calls
    TextResponse(String),
    // the agent tried to execute an invalid tool
    InvalidToolCall {
        tool_call: ToolCall,
        error: Option<String>,
    },
    // the tool call timed out
    ToolCallTimeout {
        tool_call: ToolCall,
        elapsed: std::time::Duration,
    },
    // a tool call is about to execute
    BeforeToolCall {
        tool_call: ToolCall,
    },
    // a tool call has been executed
    AfterToolCall {
        tool_call: ToolCall,
        error: Option<String>,
        result: Option<ToolOutput>,
        elapsed: std::time::Duration,
        complete_task: bool,
    },
    // the task has been completed
    TaskComplete {
        // set to true if the agent determined the task was impossible
        impossible: bool,
        // the reason why task was set as complete
        reason: Option<String>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    // event timestamp in nanoseconds
    pub timestamp: u128,
    // the actual event data
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
