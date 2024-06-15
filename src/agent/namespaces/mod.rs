use std::collections::HashMap;

use anyhow::Result;
use lazy_static::lazy_static;

use super::state::{storage::StorageType, State};

// TODO: add more namespaces of actions: fs (read only), take screenshot (multimodal), networking, move mouse, ui interactions, etc

pub(crate) mod goal;
pub(crate) mod memory;
pub(crate) mod planning;
pub(crate) mod task;

lazy_static! {
    // Available namespaces.
    pub static ref NAMESPACES: HashMap<String, fn() -> Namespace> = {
        let mut map = HashMap::new();

        map.insert("memory".to_string(), memory::get_namespace as fn() -> Namespace);
        map.insert("goal".to_string(), goal::get_namespace as fn() -> Namespace);
        map.insert("planning".to_string(), planning::get_namespace as fn() -> Namespace);
        map.insert("task".to_string(), task::get_namespace as fn() -> Namespace);

        map
    };
}

#[derive(Debug)]
pub struct StorageDescriptor {
    pub name: String,
    pub type_: StorageType,
}

impl StorageDescriptor {
    pub fn tagged(name: &str) -> Self {
        let name = name.to_string();
        let type_ = StorageType::Tagged;
        Self { name, type_ }
    }

    pub fn untagged(name: &str) -> Self {
        let name = name.to_string();
        let type_ = StorageType::Untagged;
        Self { name, type_ }
    }

    pub fn previous_current(name: &str) -> Self {
        let name = name.to_string();
        let type_ = StorageType::CurrentPrevious;
        Self { name, type_ }
    }
}

#[derive(Debug, Default)]
pub struct Namespace {
    pub name: String,
    pub description: String,
    pub actions: Vec<Box<dyn Action>>,
    pub storages: Option<Vec<StorageDescriptor>>,
}

impl Namespace {
    pub fn new(
        name: String,
        description: String,
        actions: Vec<Box<dyn Action>>,
        storages: Option<Vec<StorageDescriptor>>,
    ) -> Self {
        Self {
            name,
            description,
            actions,
            storages,
        }
    }
}

pub trait Action: std::fmt::Debug + Sync {
    fn name(&self) -> &str;
    fn attributes(&self) -> Option<HashMap<String, String>> {
        None
    }
    fn example_payload(&self) -> Option<&str> {
        None
    }

    fn description(&self) -> &str;

    fn structured_example(&self) -> String {
        let mut xml = format!("<{}", self.name());

        if let Some(attrs) = self.attributes() {
            for (name, example_value) in &attrs {
                xml += &format!(" {}=\"{}\"", name, example_value);
            }
        }
        xml += ">";

        if let Some(payload) = self.example_payload() {
            xml += payload; // TODO: escape payload?
        }

        xml += &format!("</{}>", self.name());

        xml
    }

    fn run(
        &self,
        state: &State,
        attributes: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>>;
}
