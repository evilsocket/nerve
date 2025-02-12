use std::{collections::HashMap, fmt::Display, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use super::state::{storage::StorageType, SharedState};

pub mod filesystem;
pub mod goal;
pub mod http;
pub mod memory;
pub mod planning;
pub mod rag;
pub mod reasoning;
pub mod shell;
pub mod task;
pub mod time;

lazy_static! {
    // Available namespaces.
    pub static ref NAMESPACES: IndexMap<String, fn() -> Namespace> = {
        let mut map = IndexMap::new();

        map.insert("memory".to_string(), memory::get_namespace as fn() -> Namespace);
        map.insert("reasoning".to_string(), reasoning::get_namespace as fn() -> Namespace);
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
    pub fn text(name: &str) -> Self {
        let name = name.to_string();
        let type_ = StorageType::Text;
        let predefined = None;
        Self {
            name,
            type_,
            predefined,
        }
    }

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
    pub tools: Vec<Box<dyn Tool>>,
    pub storages: Option<Vec<StorageDescriptor>>,
    pub default: bool,
}

impl Namespace {
    pub fn new_non_default(
        name: String,
        description: String,
        tools: Vec<Box<dyn Tool>>,
        storages: Option<Vec<StorageDescriptor>>,
    ) -> Self {
        let default = false;
        Self {
            name,
            description,
            tools,
            storages,
            default,
        }
    }

    pub fn new_default(
        name: String,
        description: String,
        tools: Vec<Box<dyn Tool>>,
        storages: Option<Vec<StorageDescriptor>>,
    ) -> Self {
        let default = true;
        Self {
            name,
            description,
            tools,
            storages,
            default,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ToolOutput {
    Text(String),
    Image { data: String, mime_type: String },
}

impl ToolOutput {
    pub fn text<S: Into<String>>(text: S) -> Self {
        ToolOutput::Text(text.into())
    }
    pub fn image(data: String, mime_type: String) -> Self {
        ToolOutput::Image { data, mime_type }
    }
}

impl From<String> for ToolOutput {
    fn from(text: String) -> Self {
        ToolOutput::text(text)
    }
}

impl From<&str> for ToolOutput {
    fn from(text: &str) -> Self {
        ToolOutput::text(text)
    }
}

impl Display for ToolOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolOutput::Text(text) => write!(f, "{}", text),
            ToolOutput::Image { data, mime_type } => {
                write!(f, "image: {} ({})", data, mime_type)
            }
        }
    }
}

#[async_trait]
pub trait Tool: std::fmt::Debug + Sync + Send + ToolClone {
    fn name(&self) -> &str;

    fn description(&self) -> &str;

    async fn run(
        &self,
        state: SharedState,
        attributes: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<ToolOutput>>;

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

    // optional variables used by this tool
    fn required_variables(&self) -> Option<Vec<String>> {
        None
    }

    // optional method to indicate if this tool requires user confirmation before execution
    fn requires_user_confirmation(&self) -> bool {
        false
    }

    // complete the task after execution
    fn complete_task(&self) -> bool {
        false
    }
}

// https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
// Splitting ToolClone into its own trait allows us to provide a blanket
// implementation for all compatible types, without having to implement the
// rest of Tool.  In this case, we implement it for all types that have
// 'static lifetime (*i.e.* they don't contain non-'static pointers), and
// implement both Tool and Clone.  Don't ask me how the compiler resolves
// implementing ToolClone for dyn Tool when Tool requires ToolClone;
// I have *no* idea why this works.
pub trait ToolClone {
    fn clone_box(&self) -> Box<dyn Tool>;
}

impl<T> ToolClone for T
where
    T: 'static + Tool + Clone,
{
    fn clone_box(&self) -> Box<dyn Tool> {
        Box::new(self.clone())
    }
}

// We can now implement Clone manually by forwarding to clone_box.
impl Clone for Box<dyn Tool> {
    fn clone(&self) -> Box<dyn Tool> {
        self.clone_box()
    }
}
