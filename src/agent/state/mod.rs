use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use colored::Colorize;
use metrics::Metrics;

use super::{
    generator::{Client, Message},
    namespaces::{self, Namespace},
    rag::{naive::NaiveVectorStore, Document, VectorStore},
    task::Task,
    Invocation,
};
use history::{Execution, History};
use storage::Storage;

mod history;
mod metrics;
pub(crate) mod storage;

pub struct State {
    // the task
    task: Box<dyn Task>,
    // model memories, goals and other storages
    storages: HashMap<String, Storage>,
    // available actions and execution history
    namespaces: Vec<Namespace>,
    // list of executed actions
    history: History,
    // optional rag engine
    rag: Option<Box<dyn VectorStore>>,
    // set to true when task is complete
    complete: bool,
    // runtime metrics
    pub metrics: Metrics,
}

pub type SharedState = Arc<tokio::sync::Mutex<State>>;

impl State {
    pub async fn new(
        task: Box<dyn Task>,
        embedder: Box<dyn Client>,
        max_iterations: usize,
    ) -> Result<Self> {
        let complete = false;
        let mut storages = HashMap::new();
        let history = History::new();

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

        // add RAG namespace
        let rag: Option<Box<dyn VectorStore>> = if let Some(config) = task.get_rag_config() {
            let v_store: NaiveVectorStore =
                NaiveVectorStore::from_indexed_path(embedder, &config.path).await?;

            namespaces.push(namespaces::NAMESPACES.get("rag").unwrap()());

            Some(Box::new(v_store))
        } else {
            None
        };

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
        if let Some(goal) = storages.get_mut("goal") {
            let prompt = task.to_prompt()?;
            goal.set_current(&prompt, false);
        }

        let metrics = Metrics {
            max_steps: max_iterations,
            ..Default::default()
        };

        Ok(Self {
            task,
            storages,
            history,
            namespaces,
            complete,
            metrics,
            rag,
        })
    }

    pub fn on_step(&mut self) -> Result<()> {
        self.metrics.current_step += 1;
        if self.metrics.max_steps > 0 && self.metrics.current_step >= self.metrics.max_steps {
            Err(anyhow!("maximum number of steps reached"))
        } else {
            Ok(())
        }
    }

    pub async fn rag_query(&mut self, query: &str, top_k: usize) -> Result<Vec<(Document, f64)>> {
        if let Some(rag) = &self.rag {
            rag.retrieve(query, top_k).await
        } else {
            Err(anyhow!("no RAG engine has been configured"))
        }
    }

    pub fn to_chat_history(&self, max: usize) -> Result<Vec<Message>> {
        self.history.to_chat_history(max)
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

    pub fn get_storage_mut(&mut self, name: &str) -> Result<&mut Storage> {
        if let Some(storage) = self.storages.get_mut(name) {
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
        self.complete
    }

    pub fn get_namespaces(&self) -> &Vec<Namespace> {
        &self.namespaces
    }

    pub fn add_success_to_history(&mut self, invocation: Invocation, result: Option<String>) {
        self.history
            .push(Execution::with_result(invocation, result));
    }

    pub fn add_error_to_history(&mut self, invocation: Invocation, error: String) {
        // eprintln!("[{}] -> {}", &invocation.action, error.red());
        self.history.push(Execution::with_error(invocation, error));
    }

    pub fn add_unparsed_response_to_history(&mut self, response: &str, error: String) {
        self.history
            .push(Execution::with_unparsed_response(response, error));
    }

    pub fn get_action(&self, name: &str) -> Option<Box<dyn namespaces::Action>> {
        for group in &self.namespaces {
            for action in &group.actions {
                if name == action.name() {
                    return Some(action.clone());
                }
            }
        }

        None
    }

    /*
       pub async fn execute(&mut self, invocation: Invocation) -> Result<()> {
           let action = match self.get_action(&invocation.action) {
               Some(action) => action,
               None => {
                   self.metrics.errors.unknown_actions += 1;
                   // tell the model that the action name is wrong
                   self.add_error_to_history(
                       invocation.clone(),
                       format!("'{}' is not a valid action name", invocation.action),
                   );
                   return Ok(());
               }
           };

           // validate prerequisites
           if let Err(e) = self.validate(&invocation, action) {
               self.metrics.errors.invalid_actions += 1;
               self.add_error_to_history(invocation.clone(), e.to_string());
               // not a core error, just inform the model and return
               return Ok(());
           }

           // execute the action

           // TODO: add timeout logic to invocations
           let inv = invocation.clone();
           let shared_state = get_state_fn();

           let ret = action
               .run(shared_state, invocation.attributes, invocation.payload)
               .await;
           if let Err(error) = ret {
               self.metrics.errors.errored_actions += 1;
               // tell the model about the error
               self.add_error_to_history(inv, error.to_string());
           } else {
               self.metrics.success_actions += 1;
               // tell the model about the output
               self.add_success_to_history(inv, ret.unwrap());
           }

           Ok(())
       }
    */
    pub fn on_complete(&mut self, impossible: bool, reason: Option<String>) -> Result<()> {
        // TODO: unify logging logic
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

        self.complete = true;
        Ok(())
    }
}
