use anyhow::Result;
use std::collections::HashMap;

use super::state::State;

pub(crate) mod memory;
pub(crate) mod task;

#[derive(Debug, Default)]
pub struct Namespace {
    pub name: String,
    pub description: String,
    pub actions: Vec<Box<dyn Action>>,
}

impl Namespace {
    pub fn new(name: String, description: String, actions: Vec<Box<dyn Action>>) -> Self {
        Self {
            name,
            description,
            actions,
        }
    }
}

pub trait Action: std::fmt::Debug {
    fn name(&self) -> &str;
    fn attributes(&self) -> Option<HashMap<String, String>> {
        None
    }
    fn example_payload(&self) -> Option<&str> {
        None
    }
    fn add_to_activity(&self) -> bool {
        true
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
