use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Duration};

use anyhow::Result;
use mini_rag::Embedder;
use serde::{Deserialize, Serialize};

use events::Event;
use generator::{Client, Options};
use namespaces::Action;
use serialization::xml::serialize;
use state::{SharedState, State};
use task::Task;

pub mod events;
pub mod generator;
pub mod namespaces;
pub mod serialization;
pub mod state;
pub mod task;

pub fn data_path(path: &str) -> Result<PathBuf> {
    let user_home = match simple_home_dir::home_dir() {
        Some(path) => path,
        None => return Err(anyhow!("can't get user home folder")),
    };

    let inner_path = user_home.join(".nerve").join(path);
    if !inner_path.exists() {
        log::info!("creating {} ...", inner_path.display());
        std::fs::create_dir_all(&inner_path)?;
    }

    Ok(inner_path)
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invocation {
    pub action: String,
    pub attributes: Option<HashMap<String, String>>,
    pub payload: Option<String>,
}

impl std::fmt::Display for Invocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serialize::invocation(self))
    }
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

pub struct Agent {
    events_chan: events::Sender,
    generator: Box<dyn Client>,
    state: SharedState,
    max_history: u16,
    task_timeout: Option<Duration>,
}

impl Agent {
    pub async fn new(
        events_chan: events::Sender,
        generator: Box<dyn Client>,
        embedder: Box<dyn Embedder>,
        task: Box<dyn Task>,
        max_iterations: usize,
    ) -> Result<Self> {
        // check if the model supports tools calling natively
        let tools_support = generator.check_tools_support().await?;
        if tools_support {
            log::info!("model supports tools calling natively.");
        } else {
            log::info!(
                "model does not support tools calling natively, using Nerve custom system prompt"
            );
        }

        let max_history = task.max_history_visibility();
        let task_timeout = task.get_timeout();
        let state = Arc::new(tokio::sync::Mutex::new(
            State::new(
                events_chan.clone(),
                task,
                embedder,
                max_iterations,
                tools_support,
            )
            .await?,
        ));

        Ok(Self {
            events_chan,
            generator,
            state,
            max_history,
            task_timeout,
        })
    }

    #[allow(clippy::borrowed_box)]
    pub fn validate(&self, invocation: &Invocation, action: &Box<dyn Action>) -> Result<()> {
        // validate prerequisites
        let payload_required = action.example_payload().is_some();
        let attrs_required = action.example_attributes().is_some();
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
                .example_attributes()
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

    async fn on_state_update(&self, options: &Options, refresh: bool) -> Result<()> {
        let mut opts = options.clone();
        if refresh {
            opts.system_prompt = serialization::state_to_system_prompt(&*self.state.lock().await)?;
            opts.history = self
                .state
                .lock()
                .await
                .to_chat_history(self.max_history as usize)?;
        }

        self.on_event(events::Event::StateUpdate(opts))
    }

    async fn on_empty_response(&self) {
        let mut mut_state = self.state.lock().await;
        mut_state.metrics.errors.empty_responses += 1;
        mut_state
            .add_unparsed_response_to_history("", "Do not return an empty responses.".to_string());

        self.on_event(events::Event::EmptyResponse).unwrap();
    }

    async fn on_invalid_response(&self, response: &str) {
        let mut mut_state = self.state.lock().await;
        mut_state.metrics.errors.unparsed_responses += 1;
        mut_state.add_unparsed_response_to_history(
        response,
        "I could not parse any valid actions from your response, please correct it according to the instructions.".to_string(),
    );
        self.on_event(events::Event::InvalidResponse(response.to_string()))
            .unwrap();
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
            invocation.clone(),
            error
                .clone()
                .unwrap_or(format!("'{name}' is not a valid action name")),
        );

        self.on_event(events::Event::InvalidAction { invocation, error })
            .unwrap();
    }

    async fn on_valid_action(&self) {
        self.state.lock().await.metrics.valid_actions += 1;
    }

    async fn on_timed_out_action(&self, invocation: Invocation, start: &std::time::Instant) {
        let mut mut_state = self.state.lock().await;
        mut_state.metrics.errors.timedout_actions += 1;
        // tell the model about the timeout
        mut_state.add_error_to_history(invocation.clone(), "action timed out".to_string());

        self.events_chan
            .send(events::Event::ActionTimeout {
                invocation,
                elapsed: start.elapsed(),
            })
            .unwrap();
    }

    async fn on_executed_action(
        &self,
        invocation: Invocation,
        ret: Result<Option<String>>,
        start: &std::time::Instant,
    ) {
        let mut mut_state = self.state.lock().await;
        let mut error = None;
        let mut result = None;

        if let Err(err) = ret {
            mut_state.metrics.errors.errored_actions += 1;
            // tell the model about the error
            mut_state.add_error_to_history(invocation.clone(), err.to_string());

            error = Some(err.to_string());
        } else {
            let ret = ret.unwrap();
            mut_state.metrics.success_actions += 1;
            // tell the model about the output
            mut_state.add_success_to_history(invocation.clone(), ret.clone());

            result = ret;
        }

        self.on_event(events::Event::ActionExecuted {
            invocation,
            result,
            error,
            elapsed: start.elapsed(),
        })
        .unwrap();
    }

    pub async fn get_metrics(&self) -> state::metrics::Metrics {
        self.state.lock().await.metrics.clone()
    }

    async fn prepare_step(&mut self) -> Result<Options> {
        let mut mut_state = self.state.lock().await;

        mut_state.on_step()?;

        self.on_event(events::Event::MetricsUpdate(mut_state.metrics.clone()))?;

        let system_prompt = serialization::state_to_system_prompt(&mut_state)?;
        let prompt = mut_state.to_prompt()?;
        let history = mut_state.to_chat_history(self.max_history as usize)?;
        let options = Options::new(system_prompt, prompt, history);

        Ok(options)
    }

    pub fn on_event(&self, event: Event) -> Result<()> {
        self.events_chan.send(event).map_err(|e| anyhow!(e))
    }

    pub async fn step(&mut self) -> Result<()> {
        let options = self.prepare_step().await?;

        self.on_state_update(&options, false).await?;

        // run model inference
        let (response, tool_calls) = self.generator.chat(self.state.clone(), &options).await?;

        // parse the model response into invocations
        let invocations = if tool_calls.is_empty() {
            serialization::xml::parsing::try_parse(response.trim())?
        } else {
            tool_calls
        };

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
                        self.on_executed_action(inv, ret.unwrap(), &start).await;
                    }
                }
            }

            self.on_state_update(&options, true).await?;

            // break the loop if we're done
            if self.state.lock().await.is_complete() {
                break;
            }
        }

        Ok(())
    }

    pub async fn on_end(&mut self) -> Result<()> {
        // report final metrics on exit
        let last_metrics = self.get_metrics().await;

        self.on_event(Event::MetricsUpdate(last_metrics))
    }
}
