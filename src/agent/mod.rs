use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use colored::Colorize;

use generator::{Client, Options};
use namespaces::Action;
use state::{SharedState, State};
use task::Task;

pub mod generator;
pub mod namespaces;
pub mod rag;
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
    pub with_stats: bool,
}

pub struct Agent {
    generator: Box<dyn Client>,
    state: SharedState,
    options: AgentOptions,
    max_history: u16,
}

impl Agent {
    pub async fn new(
        generator: Box<dyn Client>,
        embedder: Box<dyn Client>,
        task: Box<dyn Task>,
        options: AgentOptions,
    ) -> Result<Self> {
        let max_history = task.max_history_visibility();
        let state = Arc::new(tokio::sync::Mutex::new(
            State::new(task, embedder, options.max_iterations).await?,
        ));

        Ok(Self {
            generator,
            state,
            options,
            max_history,
        })
    }

    #[allow(clippy::borrowed_box)]
    pub fn validate(&self, invocation: &Invocation, action: &Box<dyn Action>) -> Result<()> {
        // validate prerequisites
        let payload_required = action.example_payload().is_some();
        let attrs_required = action.attributes().is_some();
        let has_payload = invocation.payload.is_some();
        let has_attributes = invocation.attributes.is_some();

        if payload_required && !has_payload {
            // payload required and not specified
            return Err(anyhow!(
                "no xml content specified for '{}'",
                invocation.action
            ));
        } else if attrs_required && !has_attributes {
            // attributes required and not specified at all
            return Err(anyhow!(
                "no xml attributes specified for '{}'",
                invocation.action
            ));
        } else if !payload_required && has_payload {
            // payload not required but specified
            return Err(anyhow!("no xml content needed for '{}'", invocation.action));
        } else if !attrs_required && has_attributes {
            // attributes not required but specified
            return Err(anyhow!(
                "no xml attributes needed for '{}'",
                invocation.action
            ));
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
                    return Err(anyhow!(
                        "no '{}' xml attribute specified for '{}'",
                        required,
                        invocation.action
                    ));
                }
            }
        }

        Ok(())
    }

    pub async fn is_done(&self) -> bool {
        self.state.lock().await.is_complete()
    }

    async fn save_if_needed(&self, options: &Options, refresh: bool) -> Result<()> {
        if let Some(prompt_path) = &self.options.save_to {
            let mut opts = options.clone();
            if refresh {
                opts.system_prompt =
                    serialization::state_to_system_prompt(&*self.state.lock().await)?;
                opts.history = self
                    .state
                    .lock()
                    .await
                    .to_chat_history(self.max_history as usize)?;
            }

            let data = if self.options.full_dump {
                format!(
                    "[SYSTEM PROMPT]\n\n{}\n\n[PROMPT]\n\n{}\n\n[CHAT]\n\n{}",
                    &options.system_prompt,
                    &options.prompt,
                    options
                        .history
                        .iter()
                        .map(|m| m.to_string())
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
        let (invocations, options) = {
            let mut mut_state = self.state.lock().await;

            mut_state.on_step()?;

            if self.options.with_stats {
                println!("\n{}\n", &mut_state.metrics);
            }

            let system_prompt = serialization::state_to_system_prompt(&mut_state)?;
            let prompt = mut_state.to_prompt()?;
            let history = mut_state.to_chat_history(self.max_history as usize)?;
            let options = Options::new(system_prompt, prompt, history);

            self.save_if_needed(&options, false).await?;

            // run model inference
            let response = self.generator.chat(&options).await?.trim().to_string();

            // parse the model response into invocations
            let invocations = serialization::xml::parsing::try_parse(&response)?;

            // nothing parsed, report the problem to the model
            if invocations.is_empty() {
                if response.is_empty() {
                    println!(
                        "{}: agent did not provide valid instructions: empty response",
                        "WARNING".bold().red(),
                    );

                    mut_state.metrics.errors.empty_responses += 1;
                    mut_state.add_unparsed_response_to_history(
                        &response,
                        "Do not return an empty responses.".to_string(),
                    );
                } else {
                    println!(
                        "{}: agent did not provide valid instructions: \n\n{}\n\n",
                        "WARNING".bold().red(),
                        response.dimmed()
                    );

                    mut_state.metrics.errors.unparsed_responses += 1;
                    mut_state.add_unparsed_response_to_history(
                    &response,
                    "I could not parse any valid actions from your response, please correct it according to the instructions.".to_string(),
                );
                }
            } else {
                mut_state.metrics.valid_responses += 1;
            }

            (invocations, options)
        };

        // for each parsed invocation
        // NOTE: the MutexGuard is purposedly captured in its own scope in order to avoid
        // deadlocks and make its lifespan clearer.
        for inv in invocations {
            // lookup action
            let action = self.state.lock().await.get_action(&inv.action);
            if action.is_none() {
                {
                    let mut mut_state = self.state.lock().await;
                    mut_state.metrics.errors.unknown_actions += 1;
                    // tell the model that the action name is wrong
                    mut_state.add_error_to_history(
                        inv.clone(),
                        format!("'{}' is not a valid action name", inv.action),
                    );
                }
            } else {
                let action = action.unwrap();
                // validate prerequisites
                let do_exec = {
                    let mut mut_state = self.state.lock().await;

                    if let Err(err) = self.validate(&inv, &action) {
                        mut_state.metrics.errors.invalid_actions += 1;
                        mut_state.add_error_to_history(inv.clone(), err.to_string());
                        false
                    } else {
                        mut_state.metrics.valid_actions += 1;
                        true
                    }
                };

                // TODO: timeout logic

                // execute
                if do_exec {
                    let ret = action
                        .run(
                            self.state.clone(),
                            inv.attributes.to_owned(),
                            inv.payload.to_owned(),
                        )
                        .await;

                    {
                        let mut mut_state = self.state.lock().await;
                        if let Err(error) = ret {
                            mut_state.metrics.errors.errored_actions += 1;
                            // tell the model about the error
                            mut_state.add_error_to_history(inv, error.to_string());
                        } else {
                            mut_state.metrics.success_actions += 1;
                            // tell the model about the output
                            mut_state.add_success_to_history(inv, ret.unwrap());
                        }
                    }
                }
            }

            self.save_if_needed(&options, true).await?;
            if self.state.lock().await.is_complete() {
                break;
            }
        }

        Ok(())
    }
}
