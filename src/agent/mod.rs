use std::collections::HashMap;

use anyhow::Result;
use colored::Colorize;

use model::{Client, Options};
use state::State;
use task::Task;

pub mod model;
pub mod namespaces;
pub mod serialization;
pub mod state;
pub mod task;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub action: String,
    pub attributes: Option<HashMap<String, String>>,
    pub payload: Option<String>,
}

impl std::hash::Hash for Invocation {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.action.hash(state);
        state.write(format!("{:?}", &self.attributes).as_bytes());
        self.payload.hash(state);
    }
}

impl Invocation {
    pub fn new(
        action: String,
        attributes: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Self {
        Self {
            action,
            attributes,
            payload,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentOptions {
    pub max_iterations: usize,
    pub save_to: Option<String>,
    pub full_dump: bool,
}

pub struct Agent {
    generator: Box<dyn Client>,
    state: State,
    options: AgentOptions,
    max_history: u16,
}

impl Agent {
    pub fn new(
        generator: Box<dyn Client>,
        task: Box<dyn Task>,
        options: AgentOptions,
    ) -> Result<Self> {
        let max_history = task.max_history_visibility();
        let state = State::new(task, options.max_iterations)?;
        Ok(Self {
            generator,
            state,
            options,
            max_history,
        })
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    fn save_if_needed(&self, options: &Options, refresh: bool) -> Result<()> {
        if let Some(prompt_path) = &self.options.save_to {
            let mut opts = options.clone();
            if refresh {
                opts.system_prompt = serialization::state_to_system_prompt(&self.state)?;
                opts.history = self.state.to_chat_history(self.max_history as usize)?;
            }

            let data = if self.options.full_dump {
                format!(
                    "[SYSTEM PROMPT]\n\n{}\n[PROMPT]\n\n{}\n[CHAT]\n\n{}",
                    &options.system_prompt,
                    &options.prompt,
                    options
                        .history
                        .iter()
                        .map(|m| format!("{:?}", &m))
                        .collect::<Vec<String>>()
                        .join("\n")
                )
            } else {
                opts.system_prompt.to_string()
            };

            std::fs::write(prompt_path, data)?;
        }

        Ok(())
    }

    pub async fn step(&mut self) -> Result<()> {
        self.state.on_next_iteration()?;

        let system_prompt = serialization::state_to_system_prompt(&self.state)?;
        let prompt = self.state.to_prompt()?;
        let history = self.state.to_chat_history(self.max_history as usize)?;

        let options = Options::new(system_prompt, prompt, history);

        self.save_if_needed(&options, false)?;

        // run model inference
        let response = self.generator.chat(&options).await?.trim().to_string();

        // parse the model response into invocations
        let invocations = serialization::xml::parsing::try_parse(&response)?;

        // nothing parsed, report the problem to the model
        if invocations.is_empty() {
            if response.is_empty() {
                self.state.add_unparsed_response_to_history(
                    &response,
                    "Do not return an empty responses.".to_string(),
                );
            } else {
                self.state.add_unparsed_response_to_history(
                    &response,
                    "I could not parse any valid actions from your response, please correct it according to the instructions.".to_string(),
                );
            }

            println!(
                "{}: agent did not provide valid instructions: {}",
                "WARNING".bold().red(),
                if response.is_empty() {
                    "empty response".dimmed().to_string()
                } else {
                    format!("\n\n{}\n\n", response.dimmed().yellow())
                }
            );
        }

        // for each parsed invocation
        for inv in invocations {
            // see if valid action and execute
            if let Err(e) = self.state.execute(inv.clone()).await {
                println!("ERROR: {}", e);
            }

            self.save_if_needed(&options, true)?;
            if self.state.is_complete() {
                break;
            }
        }

        Ok(())
    }

    pub fn get_state(&self) -> &State {
        &self.state
    }
}
