use anyhow::Result;

use super::{namespaces::NAMESPACES, state::State};
use crate::agent::{namespaces::Action, state::storage::Storage, Invocation};

mod xml;

// Serialization and deserialization strategy / format
#[derive(Debug, Clone, Default, clap::ValueEnum)]
pub enum Strategy {
    // Use XML format.
    #[default]
    XML,
}

impl Strategy {
    pub fn available_actions() -> String {
        let default_serializer = Self::default();

        let mut md = "".to_string();

        for build_fn in NAMESPACES.values() {
            let group = build_fn();
            md += &format!("## {}\n\n", group.name);
            if !group.description.is_empty() {
                md += &format!("{}\n\n", group.description);
            }
            for action in &group.actions {
                md += &format!(
                    "{} {}\n\n",
                    action.description(),
                    default_serializer.serialize_action(action)
                );
            }
        }

        md.trim().to_string()
    }

    pub fn try_parse(&self, raw: &str) -> Result<Vec<Invocation>> {
        match self {
            Strategy::XML => xml::parsing::try_parse(raw),
        }
    }

    pub fn serialize_storage(&self, storage: &Storage) -> String {
        match self {
            Strategy::XML => xml::serialize::storage(storage),
        }
    }

    pub fn serialize_action(&self, action: &Box<dyn Action>) -> String {
        match self {
            Strategy::XML => xml::serialize::action(action),
        }
    }

    pub fn serialize_invocation(&self, invocation: &Invocation) -> String {
        match self {
            Strategy::XML => xml::serialize::invocation(invocation),
        }
    }

    fn actions_for_state(&self, state: &State) -> Result<String> {
        let mut md = "".to_string();

        for group in state.get_namespaces() {
            md += &format!("## {}\n\n", group.name);
            if !group.description.is_empty() {
                md += &format!("{}\n\n", group.description);
            }
            for action in &group.actions {
                md += &format!(
                    "{} {}\n\n",
                    action.description(),
                    self.serialize_action(action)
                );
            }
        }

        Ok(md)
    }

    pub fn system_prompt_for_state(&self, state: &State) -> Result<String> {
        let task = state.get_task();
        let system_prompt = task.to_system_prompt()?;

        let mut storages = vec![];
        let mut sorted = state.get_storages();
        sorted.sort_by_key(|x| x.get_type().as_u8());

        for storage in sorted {
            storages.push(self.serialize_storage(storage));
        }

        let storages = storages.join("\n\n");
        let guidance = task
            .guidance()?
            .into_iter()
            .map(|s| format!("- {}", s))
            .collect::<Vec<String>>()
            .join("\n");

        let available_actions = if state.native_tools_support {
            // model supports tool calls, no need to add actions to the system prompt
            "".to_string()
        } else {
            // model does not support tool calls, we need to provide the actions in its system prompt
            include_str!("actions.prompt").to_owned() + "\n" + &self.actions_for_state(state)?
        };

        let iterations = if state.metrics.max_steps > 0 {
            format!(
                "You are currently at step {} of a maximum of {}.",
                state.metrics.current_step + 1,
                state.metrics.max_steps
            )
        } else {
            "".to_string()
        };

        Ok(format!(
            include_str!("system.prompt"),
            iterations = iterations,
            system_prompt = system_prompt,
            storages = storages,
            available_actions = available_actions,
            guidance = guidance,
        ))
    }
}
