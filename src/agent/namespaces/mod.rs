use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use indexmap::IndexMap;
use lazy_static::lazy_static;

use super::state::{storage::StorageType, SharedState};

// TODO: add more namespaces of actions: take screenshot (multimodal), move mouse, ui interactions, etc

pub(crate) mod filesystem;
pub(crate) mod goal;
pub(crate) mod http;
pub(crate) mod memory;
pub(crate) mod planning;
pub(crate) mod rag;
pub(crate) mod shell;
pub(crate) mod task;
pub(crate) mod time;

lazy_static! {
    // Available namespaces.
    pub static ref NAMESPACES: IndexMap<String, fn() -> Namespace> = {
        let mut map = IndexMap::new();

        map.insert("memory".to_string(), memory::get_namespace as fn() -> Namespace);
        map.insert("time".to_string(), time::get_namespace as fn() -> Namespace);
        map.insert("goal".to_string(), goal::get_namespace as fn() -> Namespace);
        map.insert("planning".to_string(), planning::get_namespace as fn() -> Namespace);
        map.insert("task".to_string(), task::get_namespace as fn() -> Namespace);
        map.insert("filesystem".to_string(), filesystem::get_namespace as fn() -> Namespace);
        map.insert("rag".to_string(), rag::get_namespace as fn() -> Namespace);
        map.insert("http".to_string(), http::get_namespace as fn() -> Namespace);
        map.insert("shell".to_string(), shell::get_namespace as fn() -> Namespace);

        map
    };
}

#[derive(Debug, Default)]
pub struct StorageDescriptor {
    pub name: String,
    pub type_: StorageType,
    pub predefined: Option<HashMap<String, String>>,
}

#[allow(dead_code)]
impl StorageDescriptor {
    pub fn tagged(name: &str) -> Self {
        let name = name.to_string();
        let type_ = StorageType::Tagged;
        let predefined = None;
        Self {
            name,
            type_,
            predefined,
        }
    }

    pub fn untagged(name: &str) -> Self {
        let name = name.to_string();
        let type_ = StorageType::Untagged;
        let predefined = None;
        Self {
            name,
            type_,
            predefined,
        }
    }

    pub fn previous_current(name: &str) -> Self {
        let name = name.to_string();
        let type_ = StorageType::CurrentPrevious;
        let predefined = None;
        Self {
            name,
            type_,
            predefined,
        }
    }

    pub fn completion(name: &str) -> Self {
        let name = name.to_string();
        let type_ = StorageType::Completion;
        let predefined = None;
        Self {
            name,
            type_,
            predefined,
        }
    }

    pub fn time(name: &str) -> Self {
        let name = name.to_string();
        let type_ = StorageType::Time;
        let predefined = None;
        Self {
            name,
            type_,
            predefined,
        }
    }

    pub fn predefine(mut self, what: HashMap<String, String>) -> Self {
        self.predefined = Some(what);
        self
    }
}

#[derive(Debug, Default)]
pub struct Namespace {
    pub name: String,
    pub description: String,
    pub actions: Vec<Box<dyn Action>>,
    pub storages: Option<Vec<StorageDescriptor>>,
    pub default: bool,
}

impl Namespace {
    pub fn new_non_default(
        name: String,
        description: String,
        actions: Vec<Box<dyn Action>>,
        storages: Option<Vec<StorageDescriptor>>,
    ) -> Self {
        let default = false;
        Self {
            name,
            description,
            actions,
            storages,
            default,
        }
    }

    pub fn new_default(
        name: String,
        description: String,
        actions: Vec<Box<dyn Action>>,
        storages: Option<Vec<StorageDescriptor>>,
    ) -> Self {
        let default = true;
        Self {
            name,
            description,
            actions,
            storages,
            default,
        }
    }
}

#[async_trait]
pub(crate) trait Action: std::fmt::Debug + Sync + Send + ActionClone {
    fn name(&self) -> &str;

    fn description(&self) -> &str;

    async fn run(
        &self,
        state: SharedState,
        attributes: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>>;

    // optional execution timeout
    fn timeout(&self) -> Option<Duration> {
        None
    }

    // optional example attributes
    fn example_attributes(&self) -> Option<HashMap<String, String>> {
        None
    }

    // optional example payload
    fn example_payload(&self) -> Option<&str> {
        None
    }

    // optional variables used by this action
    fn required_variables(&self) -> Option<Vec<String>> {
        None
    }

    // optional method to indicate if this action requires user confirmation before execution
    fn requires_user_confirmation(&self) -> bool {
        false
    }
}

// https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
// Splitting ActionClone into its own trait allows us to provide a blanket
// implementation for all compatible types, without having to implement the
// rest of Action.  In this case, we implement it for all types that have
// 'static lifetime (*i.e.* they don't contain non-'static pointers), and
// implement both Action and Clone.  Don't ask me how the compiler resolves
// implementing ActionClone for dyn Action when Action requires ActionClone;
// I have *no* idea why this works.
pub(crate) trait ActionClone {
    fn clone_box(&self) -> Box<dyn Action>;
}

impl<T> ActionClone for T
where
    T: 'static + Action + Clone,
{
    fn clone_box(&self) -> Box<dyn Action> {
        Box::new(self.clone())
    }
}

// We can now implement Clone manually by forwarding to clone_box.
impl Clone for Box<dyn Action> {
    fn clone(&self) -> Box<dyn Action> {
        self.clone_box()
    }
}
