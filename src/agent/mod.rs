use std::{
    collections::HashMap,
    io::{self, Write},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use anyhow::Result;
use mini_rag::Embedder;
use serde::{Deserialize, Serialize};

use events::Event;
use generator::{
    history::{ChatHistory, ConversationWindow},
    ChatOptions, ChatResponse, Client,
};
use namespaces::Action;
use state::{SharedState, State};
use task::Task;

pub mod events;
pub mod generator;
pub mod namespaces;
pub mod serialization;
pub mod state;
pub mod task;

pub fn get_user_input(prompt: &str) -> String {
    print!("\n{}", prompt);
    let _ = io::stdout().flush();

    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_goes_into_input_above) => {}
        Err(_no_updates_is_fine) => {}
    }
    println!();
    input.trim().to_string()
}

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

    pub fn as_function_call_string(&self) -> String {
        let mut parts = vec![];

        if let Some(payload) = &self.payload {
            parts.push(payload.to_owned());
        }

        if let Some(attributes) = &self.attributes {
            for (name, value) in attributes {
                parts.push(format!("{}={}", name, value))
            }
        }

        format!("{}({})", &self.action, parts.join(", "))
    }
}

pub struct Agent {
    events_chan: events::Sender,
    generator: Box<dyn Client>,
    state: SharedState,
    task_timeout: Option<Duration>,
    conversation_window: ConversationWindow,

    serializer: serialization::Strategy,
    use_native_tools_format: bool,
    user_only: bool,
}

impl Agent {
    pub async fn new(
        events_chan: events::Sender,
        generator: Box<dyn Client>,
        embedder: Box<dyn Embedder>,
        task: Box<dyn Task>,
        serializer: serialization::Strategy,
        conversation_window: ConversationWindow,
        force_strategy: bool,
        user_only: bool,
        max_iterations: usize,
    ) -> Result<Self> {
        let use_native_tools_format = if force_strategy {
            log::info!("using {:?} serialization strategy", &serializer);
            false
        } else {
            // check if the model supports tools calling natively
            match generator.check_native_tools_support().await? {
                true => {
                    log::debug!("model supports tools calling natively.");
                    true
                }
                false => {
                    log::info!("model does not support tools calling natively, using Nerve custom system prompt");
                    false
                }
            }
        };

        let task_timeout = task.get_timeout();
        let state = Arc::new(tokio::sync::Mutex::new(
            State::new(
                events_chan.clone(),
                task,
                embedder,
                max_iterations,
                use_native_tools_format,
            )
            .await?,
        ));

        Ok(Self {
            events_chan,
            generator,
            state,
            task_timeout,
            use_native_tools_format,
            user_only,
            serializer,
            conversation_window,
        })
    }

    #[allow(clippy::borrowed_box)]
    pub fn validate(&self, invocation: &mut Invocation, action: &Box<dyn Action>) -> Result<()> {
        // validate prerequisites
        let payload_required = action.example_payload().is_some();
        let attrs_required = action.example_attributes().is_some();
        let mut has_payload = invocation.payload.is_some();
        let mut has_attributes = invocation.attributes.is_some();

        // sometimes when the tool expects a json payload, the model returns it as separate arguments
        // in this case we need to convert it back to a single json string
        if (payload_required && !has_payload) && (!attrs_required && has_attributes) {
            log::warn!(
                "model returned the payload as separate arguments, converting back to payload"
            );
            invocation.payload = Some(serde_json::to_string(&invocation.attributes).unwrap());
            invocation.attributes = None;
            has_payload = true;
            has_attributes = false;
        }

        if payload_required && !has_payload {
            // payload required and not specified
            return Err(anyhow!("no content specified for '{}'", invocation.action));
        } else if attrs_required && !has_attributes {
            // attributes required and not specified at all
            return Err(anyhow!(
                "no attributes specified for '{}'",
                invocation.action
            ));
        } else if !payload_required && has_payload {
            // payload not required but specified
            return Err(anyhow!("no content needed for '{}'", invocation.action));
        } else if !attrs_required && has_attributes {
            // attributes not required but specified
            return Err(anyhow!("no attributes needed for '{}'", invocation.action));
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
                        "no '{}' attribute specified for '{}'",
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

    async fn on_state_update(&self, options: &ChatOptions, refresh: bool) -> Result<()> {
        let mut opts = options.clone();
        if refresh {
            opts.system_prompt = Some(
                self.serializer
                    .system_prompt_for_state(&*self.state.lock().await)?,
            );

            let messages = self.state.lock().await.to_chat_history(&self.serializer)?;

            opts.history = ChatHistory::create(messages, self.conversation_window);
        }

        self.on_event(events::Event::StateUpdate(opts))
    }

    // TODO: move these feedback strings to a common place

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
        action: &Box<dyn Action>,
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
            complete_task: action.complete_task(),
        })
        .unwrap();
    }

    async fn on_completion(&self, response: &ChatResponse) {
        // update tokens usage if available from the generator
        if let Some(usage) = &response.usage {
            let mut mut_state = self.state.lock().await;
            mut_state.metrics.usage.last_input_tokens = usage.input_tokens;
            mut_state.metrics.usage.last_output_tokens = usage.output_tokens;
            mut_state.metrics.usage.total_input_tokens += usage.input_tokens;
            mut_state.metrics.usage.total_output_tokens += usage.output_tokens;
        }
    }

    pub async fn get_metrics(&self) -> state::metrics::Metrics {
        self.state.lock().await.metrics.clone()
    }

    async fn prepare_step(&mut self) -> Result<ChatOptions> {
        let mut mut_state = self.state.lock().await;

        mut_state.on_step()?;

        self.on_event(events::Event::MetricsUpdate(mut_state.metrics.clone()))?;

        let system_prompt = self.serializer.system_prompt_for_state(&mut_state)?;
        let prompt = mut_state.to_prompt()?;

        let (system_prompt, prompt) = if self.user_only {
            // combine with user prompt for models like the openai/o1 family
            (None, format!("{system_prompt}\n\n{prompt}"))
        } else {
            (Some(system_prompt), prompt)
        };

        let history = mut_state.to_chat_history(&self.serializer)?;
        let options = ChatOptions::new(system_prompt, prompt, history, self.conversation_window);

        Ok(options)
    }

    pub fn on_event(&self, event: Event) -> Result<()> {
        self.events_chan.send(event).map_err(|e| anyhow!(e))
    }

    pub async fn step(&mut self) -> Result<()> {
        let options = self.prepare_step().await?;

        self.on_state_update(&options, false).await?;

        // run model inference
        let response = self.generator.chat(self.state.clone(), &options).await?;

        // update tokens usage
        self.on_completion(&response).await;

        // parse the model response into invocations
        let invocations = if self.use_native_tools_format && response.invocations.is_empty() {
            // no tool calls, attempt to parse the content anyway
            self.serializer
                .try_parse(response.content.trim())
                .unwrap_or_default()
        } else if !self.use_native_tools_format {
            // use our own parsing strategy
            self.serializer.try_parse(response.content.trim())?
        } else {
            response.invocations
        };

        // nothing parsed, report the problem to the model
        if invocations.is_empty() {
            if response.content.is_empty() {
                self.on_empty_response().await;
            } else {
                self.on_invalid_response(&response.content).await;
            }
        } else {
            self.on_valid_response().await;
        }

        // for each parsed invocation
        for mut inv in invocations {
            // lookup action
            let action = self.state.lock().await.get_action(&inv.action);
            if action.is_none() {
                self.on_invalid_action(inv.clone(), None).await;
            } else {
                // validate prerequisites
                let action = action.unwrap();
                if let Err(err) = self.validate(&mut inv, &action) {
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

                    let mut execute = true;

                    if action.requires_user_confirmation() {
                        log::warn!("user confirmation required");

                        let start = std::time::Instant::now();
                        let mut inp = "nope".to_string();
                        while !inp.is_empty() && inp != "n" && inp != "y" {
                            inp =
                                get_user_input(&format!("{} [Yn] ", inv.as_function_call_string()))
                                    .to_ascii_lowercase();
                        }

                        if inp == "n" {
                            log::warn!("invocation rejected by user");
                            self.on_executed_action(
                                &action,
                                inv.clone(),
                                Err(anyhow!("rejected by user".to_owned())),
                                &start,
                            )
                            .await;

                            execute = false;
                        }
                    }

                    if execute {
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
                            self.on_executed_action(&action, inv, ret.unwrap(), &start)
                                .await;
                        }

                        if action.complete_task() {
                            log::info!("! task complete");
                            self.state.lock().await.on_complete(false, None)?;
                        }
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
