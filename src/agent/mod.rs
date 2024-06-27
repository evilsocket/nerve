use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::Result;
use colored::Colorize;

use generator::{Client, Options};
use mini_rag::Embedder;
use namespaces::Action;
use state::{SharedState, State};
use task::Task;

pub mod generator;
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
    pub with_stats: bool,
}

pub struct Agent {
    generator: Box<dyn Client>,
    state: SharedState,
    options: AgentOptions,
    max_history: u16,
    task_timeout: Option<Duration>,
}

impl Agent {
    pub async fn new(
        generator: Box<dyn Client>,
        embedder: Box<dyn Embedder>,
        task: Box<dyn Task>,
        options: AgentOptions,
    ) -> Result<Self> {
        let max_history = task.max_history_visibility();
        let task_timeout = task.get_timeout();
        let state = Arc::new(tokio::sync::Mutex::new(
            State::new(task, embedder, options.max_iterations).await?,
        ));

        Ok(Self {
            generator,
            state,
            options,
            max_history,
            task_timeout,
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

    async fn on_empty_response(&self) {
        println!(
            "{}: agent did not provide valid instructions: empty response",
            "WARNING".bold().red(),
        );

        let mut mut_state = self.state.lock().await;
        mut_state.metrics.errors.empty_responses += 1;
        mut_state
            .add_unparsed_response_to_history("", "Do not return an empty responses.".to_string());
    }

    async fn on_invalid_response(&self, response: &str) {
        println!(
            "{}: agent did not provide valid instructions: \n\n{}\n\n",
            "WARNING".bold().red(),
            response.dimmed()
        );

        let mut mut_state = self.state.lock().await;
        mut_state.metrics.errors.unparsed_responses += 1;
        mut_state.add_unparsed_response_to_history(
        response,
        "I could not parse any valid actions from your response, please correct it according to the instructions.".to_string(),
    );
    }

    async fn on_valid_response(&self) {
        self.state.lock().await.metrics.valid_responses += 1;
    }

    async fn on_invalid_action(&self, invocation: Invocation, error: Option<String>) {
        let mut mut_state = self.state.lock().await;
        mut_state.metrics.errors.unknown_actions += 1;
        // tell the model that the action name is wrong
        let name = invocation.action.clone();

        mut_state.add_error_to_history(
            invocation,
            error.unwrap_or(format!("'{name}' is not a valid action name")),
        );
    }

    async fn on_valid_action(&self) {
        self.state.lock().await.metrics.valid_actions += 1;
    }

    async fn on_timed_out_action(&self, invocation: Invocation, start: &std::time::Instant) {
        println!(
            "{}: action '{}' timed out after {:?}",
            "WARNING".bold().yellow(),
            &invocation.action,
            start.elapsed()
        );
        let mut mut_state = self.state.lock().await;
        mut_state.metrics.errors.timedout_actions += 1;
        // tell the model about the timeout
        mut_state.add_error_to_history(invocation, "action timed out".to_string());
    }

    async fn on_executed_action(&self, invocation: Invocation, ret: Result<Option<String>>) {
        let mut mut_state = self.state.lock().await;
        if let Err(error) = ret {
            mut_state.metrics.errors.errored_actions += 1;
            // tell the model about the error
            mut_state.add_error_to_history(invocation, error.to_string());
        } else {
            mut_state.metrics.success_actions += 1;
            // tell the model about the output
            mut_state.add_success_to_history(invocation, ret.unwrap());
        }
    }

    pub async fn get_metrics(&self) -> state::metrics::Metrics {
        self.state.lock().await.metrics.clone()
    }

    async fn prepare_step(&mut self) -> Result<Options> {
        let mut mut_state = self.state.lock().await;

        mut_state.on_step()?;

        if self.options.with_stats {
            println!("\n{}\n", &mut_state.metrics);
        }

        let system_prompt = serialization::state_to_system_prompt(&mut_state)?;
        let prompt = mut_state.to_prompt()?;
        let history = mut_state.to_chat_history(self.max_history as usize)?;
        let options = Options::new(system_prompt, prompt, history);

        Ok(options)
    }

    pub async fn step(&mut self) -> Result<()> {
        let options = self.prepare_step().await?;

        self.save_if_needed(&options, false).await?;

        // run model inference
        let response = self.generator.chat(&options).await?.trim().to_string();

        // parse the model response into invocations
        let invocations = serialization::xml::parsing::try_parse(&response)?;

        // nothing parsed, report the problem to the model
        if invocations.is_empty() {
            if response.is_empty() {
                self.on_empty_response().await;
            } else {
                self.on_invalid_response(&response).await;
            }
        } else {
            self.on_valid_response().await;
        }

        // for each parsed invocation
        for inv in invocations {
            // lookup action
            let action = self.state.lock().await.get_action(&inv.action);
            if action.is_none() {
                self.on_invalid_action(inv.clone(), None).await;
            } else {
                // validate prerequisites
                let action = action.unwrap();
                if let Err(err) = self.validate(&inv, &action) {
                    self.on_invalid_action(inv.clone(), Some(err.to_string()))
                        .await;
                } else {
                    self.on_valid_action().await;

                    // determine if we have a timeout
                    let timeout = if let Some(action_tm) = action.timeout().as_ref() {
                        *action_tm
                    } else if let Some(task_tm) = self.task_timeout.as_ref() {
                        *task_tm
                    } else {
                        // one month by default :D
                        Duration::from_secs(60 * 60 * 24 * 30)
                    };

                    // println!("{} timeout={:?}", action.name(), &timeout);

                    // execute with timeout
                    let start = std::time::Instant::now();
                    let ret = tokio::time::timeout(
                        timeout,
                        action.run(
                            self.state.clone(),
                            inv.attributes.to_owned(),
                            inv.payload.to_owned(),
                        ),
                    )
                    .await;

                    if ret.is_err() {
                        self.on_timed_out_action(inv, &start).await;
                    } else {
                        self.on_executed_action(inv, ret.unwrap()).await;
                    }
                }
            }

            self.save_if_needed(&options, true).await?;

            // break the loop if we're done
            if self.state.lock().await.is_complete() {
                break;
            }
        }

        Ok(())
    }
}
