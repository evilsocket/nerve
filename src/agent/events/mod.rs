use serde::{Deserialize, Serialize};

mod channel;

pub(crate) use channel::*;

use super::{
    generator::ChatOptions,
    state::{metrics::Metrics, storage::StorageType},
    Invocation,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) enum Event {
    MetricsUpdate(Metrics),
    StorageUpdate {
        storage_name: String,
        storage_type: StorageType,
        key: String,
        prev: Option<String>,
        new: Option<String>,
    },
    StateUpdate(ChatOptions),
    EmptyResponse,
    InvalidResponse(String),
    InvalidAction {
        invocation: Invocation,
        error: Option<String>,
    },
    ActionTimeout {
        invocation: Invocation,
        elapsed: std::time::Duration,
    },
    ActionExecuted {
        invocation: Invocation,
        error: Option<String>,
        result: Option<String>,
        elapsed: std::time::Duration,
    },
    TaskComplete {
        impossible: bool,
        reason: Option<String>,
    },
}
