use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};

use anyhow::Result;
use colored::Colorize;

use super::{
    model::Message,
    namespaces::{self, Namespace},
    task::Task,
    Invocation,
};
use history::{Execution, History};
use storage::Storage;

mod history;
pub(crate) mod storage;

#[derive(Debug)]
pub struct State {
    // the task
    task: Box<dyn Task>,
    // current iteration and max
    pub curr_iter: usize,
    pub max_iters: usize,
    // model memories, goals and other storages
    storages: HashMap<String, Storage>,
    // available actions and execution history
    namespaces: Vec<Namespace>,
    // list of executed actions
    history: Mutex<History>,
    // set to true when task is complete
    complete: AtomicBool,
}

impl State {
    pub fn new(task: Box<dyn Task>, max_iterations: usize) -> Result<Self> {
        let complete = AtomicBool::new(false);
        let mut storages = HashMap::new();
        let history = Mutex::new(History::new());

        let mut namespaces = vec![];
        let using = task.namespaces();

        if let Some(mut using) = using {
            let wild_card_idx = using.iter().position(|n| n == "*");
            if let Some(wild_card_idx) = wild_card_idx {
                // wildcard used, add all default namespaces and remove it from 'using'
                using.remove(wild_card_idx);
                for build_fn in namespaces::NAMESPACES.values() {
                    let ns = build_fn();
                    if ns.default {
                        namespaces.push(ns);
                    }
                }
            }

            // add only task defined namespaces
            for used_ns_name in &using {
                if let Some(build_fn) = namespaces::NAMESPACES.get(used_ns_name) {
                    let ns = build_fn();
                    namespaces.push(ns);
                } else {
                    return Err(anyhow!("no namespace '{}' defined", used_ns_name));
                }
            }
        } else {
            // add all default namespaces
            for build_fn in namespaces::NAMESPACES.values() {
                let ns = build_fn();
                if ns.default {
                    namespaces.push(ns);
                }
            }
        }

        // add task defined actions
        namespaces.append(&mut task.get_functions());

        // if any namespace requires a specific storage, create it
        for namespace in &namespaces {
            if let Some(ns_storages) = &namespace.storages {
                for storage in ns_storages {
                    // not created yet
                    if !storages.contains_key(&storage.name) {
                        storages.insert(
                            storage.name.to_string(),
                            Storage::new(&storage.name, storage.type_),
                        );
                    }
                }
            }
        }

        // println!("storages={:?}", &storages);

        // if the goal namespace is enabled, set the current goal
        if let Some(goal) = storages.get("goal") {
            let prompt = task.to_prompt()?;
            goal.set_current(&prompt, false);
        }

        Ok(Self {
            task,
            storages,
            history,
            namespaces,
            complete,
            max_iters: max_iterations,
            curr_iter: 0,
        })
    }

    pub fn on_next_iteration(&mut self) -> Result<()> {
        self.curr_iter += 1;
        if self.max_iters > 0 && self.curr_iter >= self.max_iters {
            Err(anyhow!("maximum number of iterations reached"))
        } else {
            Ok(())
        }
    }

    pub fn to_chat_history(&self, max: usize) -> Result<Vec<Message>> {
        self.history.lock().unwrap().to_chat_history(max)
    }

    #[allow(clippy::borrowed_box)]
    pub fn get_task(&self) -> &Box<dyn Task> {
        &self.task
    }

    pub fn get_storages(&self) -> Vec<&Storage> {
        self.storages.values().collect()
    }

    pub fn get_storage(&self, name: &str) -> Result<&Storage> {
        if let Some(storage) = self.storages.get(name) {
            Ok(storage)
        } else {
            println!("WARNING: requested storage '{name}' not found.");
            Err(anyhow!("storage {name} not found"))
        }
    }

    pub fn to_prompt(&self) -> Result<String> {
        self.task.to_prompt()
    }

    pub fn is_complete(&self) -> bool {
        self.complete.load(Ordering::SeqCst)
    }

    pub fn get_namespaces(&self) -> &Vec<Namespace> {
        &self.namespaces
    }

    pub fn get_used_namespaces_names(&self) -> Vec<String> {
        self.namespaces
            .iter()
            .map(|n| n.name.to_string().to_lowercase())
            .collect()
    }

    pub fn add_success_to_history(&self, invocation: Invocation, result: Option<String>) {
        if let Ok(mut guard) = self.history.lock() {
            guard.push(Execution::with_result(invocation, result));
        }
    }

    pub fn add_error_to_history(&self, invocation: Invocation, error: String) {
        if let Ok(mut guard) = self.history.lock() {
            // eprintln!("[{}] -> {}", &invocation.action, error.red());
            guard.push(Execution::with_error(invocation, error));
        }
    }

    pub fn add_unparsed_response_to_history(&self, response: &str, error: String) {
        if let Ok(mut guard) = self.history.lock() {
            guard.push(Execution::with_unparsed_response(response, error));
        }
    }

    #[allow(clippy::borrowed_box)]
    fn get_action(&self, name: &str) -> Option<&Box<dyn namespaces::Action>> {
        for group in &self.namespaces {
            for action in &group.actions {
                if name == action.name() {
                    return Some(action);
                }
            }
        }

        None
    }

    pub async fn execute(&self, invocation: Invocation) -> Result<()> {
        if let Some(action) = self.get_action(&invocation.action) {
            // validate prerequisites
            let payload_required = action.example_payload().is_some();
            let attrs_required = action.attributes().is_some();
            let has_payload = invocation.payload.is_some();
            let has_attributes = invocation.attributes.is_some();

            if payload_required && !has_payload {
                // payload required and not specified
                self.add_error_to_history(
                    invocation.clone(),
                    format!("no xml content specified for '{}'", invocation.action),
                );
                return Ok(());
            } else if attrs_required && !has_attributes {
                // attributes required and not specified at all
                self.add_error_to_history(
                    invocation.clone(),
                    format!("no xml attributes specified for '{}'", invocation.action),
                );
                return Ok(());
            } else if !payload_required && has_payload {
                // payload not required but specified
                self.add_error_to_history(
                    invocation.clone(),
                    format!("no xml content needed for '{}'", invocation.action),
                );
                return Ok(());
            } else if !attrs_required && has_attributes {
                // attributes not required but specified
                self.add_error_to_history(
                    invocation.clone(),
                    format!("no xml attributes needed for '{}'", invocation.action),
                );
                return Ok(());
            }

            if attrs_required {
                // validate each required attribute
                let required_attrs: Vec<String> = action
                    .attributes()
                    .unwrap()
                    .keys()
                    .map(|s| s.to_owned())
                    .collect();
                let passed_attrs: Vec<String> = invocation
                    .clone()
                    .attributes
                    .unwrap()
                    .keys()
                    .map(|s| s.to_owned())
                    .collect();

                for required in required_attrs {
                    if !passed_attrs.contains(&required) {
                        self.add_error_to_history(
                            invocation.clone(),
                            format!(
                                "no '{}' xml attribute specified for '{}'",
                                required, invocation.action
                            ),
                        );
                        return Ok(());
                    }
                }
            }

            // execute the action
            let inv = invocation.clone();
            let ret = action.run(self, invocation.attributes, invocation.payload);
            if let Err(error) = ret {
                // tell the model about the error
                self.add_error_to_history(inv, error.to_string());
            } else {
                // tell the model about the output
                self.add_success_to_history(inv, ret.unwrap());
            }
        } else {
            // tell the model that the action name is wrong
            self.add_error_to_history(
                invocation.clone(),
                format!("'{}' is not a valid action name", invocation.action),
            );
        }

        Ok(())
    }

    pub fn on_complete(&self, impossible: bool, reason: Option<String>) -> Result<()> {
        if impossible {
            println!(
                "\n{}: '{}'",
                "task is impossible".bold().red(),
                if let Some(r) = &reason {
                    r
                } else {
                    "no reason provided"
                }
            );
        } else {
            println!(
                "\n{}: '{}'",
                "task complete".bold().green(),
                if let Some(r) = &reason {
                    r
                } else {
                    "no reason provided"
                }
            );
        }

        self.complete.store(true, Ordering::SeqCst);
        Ok(())
    }
}
