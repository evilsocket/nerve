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

use events::{Event, EventType, StateUpdate};
use generator::{
    history::{ChatHistory, ConversationWindow},
    ChatOptions, ChatResponse, Client,
};
use namespaces::{Tool, ToolOutput};
use state::{SharedState, State};
use task::{eval::Evaluator, Task};

use crate::Control;

pub mod events;
pub mod generator;
pub mod namespaces;
pub mod serialization;
pub mod state;
pub mod task;
pub mod workflow;

pub fn get_user_input(prompt: &str) -> String {
    // TODO: handle this better and add to the events logic
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
pub struct ToolCall {
    pub tool_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub named_arguments: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub argument: Option<String>,
}

impl std::hash::Hash for ToolCall {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tool_name.hash(state);
        state.write(format!("{:?}", &self.named_arguments).as_bytes());
        self.argument.hash(state);
    }
}

impl ToolCall {
    pub fn new(
        tool_name: String,
        attributes: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Self {
        Self {
            tool_name,
            named_arguments: attributes,
            argument: payload,
        }
    }

    pub fn as_function_call_string(&self) -> String {
        let mut parts = vec![];

        if let Some(payload) = &self.argument {
            parts.push(payload.to_owned());
        }

        if let Some(attributes) = &self.named_arguments {
            for (name, value) in attributes {
                parts.push(format!("{}={}", name, value))
            }
        }

        format!("{}({})", &self.tool_name, parts.join(", "))
    }
}

pub struct Config {
    pub serializer: serialization::Strategy,
    pub conversation_window: ConversationWindow,
    pub force_strategy: bool,
    pub user_only: bool,
    pub max_iterations: usize,
    pub cot_tags: Vec<String>,
    pub remote: Control,
}

pub struct TaskSpecs {
    pub timeout: Option<Duration>,
    pub evaluator: Option<Evaluator>,
    pub working_directory: Option<String>,
}

pub struct Agent {
    // were events are sent to
    events_chan: events::Sender,
    // the LLM client
    generator: Box<dyn Client>,
    // the state object
    state: SharedState,
    // task specs
    task_specs: TaskSpecs,
    // agent configuration
    config: Config,
    // whether to use native tools format
    use_native_tools_format: bool,
}

impl Agent {
    pub async fn new(
        events_chan: events::Sender,
        generator: Box<dyn Client>,
        embedder: Box<dyn Embedder>,
        task: Box<dyn Task>,
        mut config: Config,
    ) -> Result<Self> {
        // check if the model supports tools calling and system prompt natively
        let supported_features = generator.check_supported_features().await?;

        let use_native_tools_format = if config.force_strategy {
            log::info!("using {:?} serialization strategy", &config.serializer);
            false
        } else {
            match supported_features.tools {
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

        config.user_only = if !config.user_only && !supported_features.system_prompt {
            log::info!("model does not support system prompt, forcing user prompt");
            true
        } else {
            // leave whatever the user set
            config.user_only
        };

        let task_specs = TaskSpecs {
            timeout: task.get_timeout(),
            evaluator: task.get_evaluator(),
            working_directory: task.get_working_directory(),
        };

        let state = Arc::new(tokio::sync::Mutex::new(
            State::new(
                events_chan.clone(),
                task,
                embedder,
                config.max_iterations,
                use_native_tools_format,
            )
            .await?,
        ));

        Ok(Self {
            events_chan,
            generator,
            state,
            task_specs,
            use_native_tools_format,
            config,
        })
    }

    #[allow(clippy::borrowed_box)]
    pub fn validate(&self, tool_call: &mut ToolCall, tool: &Box<dyn Tool>) -> Result<()> {
        // validate prerequisites
        let payload_required = tool.example_payload().is_some();
        let attrs_required = tool.example_attributes().is_some();
        let mut has_payload = tool_call.argument.is_some();
        let mut has_attributes = tool_call.named_arguments.is_some();

        // sometimes when the tool expects a json payload, the model returns it as separate arguments
        // in this case we need to convert it back to a single json string
        if (payload_required && !has_payload) && (!attrs_required && has_attributes) {
            log::warn!(
                "model returned the payload as separate arguments, converting back to payload"
            );
            tool_call.argument = Some(serde_json::to_string(&tool_call.named_arguments).unwrap());
            tool_call.named_arguments = None;
            has_payload = true;
            has_attributes = false;
        }

        if payload_required && !has_payload {
            // payload required and not specified
            return Err(anyhow!(
                "no content specified for '{}'",
                tool_call.tool_name
            ));
        } else if attrs_required && !has_attributes {
            // attributes required and not specified at all
            return Err(anyhow!(
                "no attributes specified for '{}'",
                tool_call.tool_name
            ));
        } else if !payload_required && has_payload {
            // payload not required but specified
            return Err(anyhow!("no content needed for '{}'", tool_call.tool_name));
        } else if !attrs_required && has_attributes {
            // attributes not required but specified
            return Err(anyhow!(
                "no attributes needed for '{}'",
                tool_call.tool_name
            ));
        }

        if attrs_required {
            // validate each required attribute
            let required_attrs: Vec<String> = tool
                .example_attributes()
                .unwrap()
                .keys()
                .map(|s| s.to_owned())
                .collect();
            let passed_attrs: Vec<String> = tool_call
                .clone()
                .named_arguments
                .unwrap()
                .keys()
                .map(|s| s.to_owned())
                .collect();

            for required in required_attrs {
                if !passed_attrs.contains(&required) {
                    return Err(anyhow!(
                        "no '{}' attribute specified for '{}'",
                        required,
                        tool_call.tool_name
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
        let mut state_update = StateUpdate {
            chat: options.clone(),
            globals: task::variables::get_variables(),
            variables: self.state.lock().await.get_variables().clone(),
        };

        if refresh {
            state_update.chat.system_prompt = Some(
                self.config
                    .serializer
                    .system_prompt_for_state(&*self.state.lock().await)?,
            );

            let messages = self
                .state
                .lock()
                .await
                .to_chat_history(&self.config.serializer)?;

            state_update.chat.history =
                ChatHistory::create(messages, self.config.conversation_window);
        }

        // if there was a state change
        if refresh {
            // if this task has an evaluation strategy
            if let Some(task_evaluation) = &self.task_specs.evaluator {
                // run it
                let evaluation = task_evaluation
                    .evaluate(&state_update, &self.task_specs.working_directory)
                    .await;
                if let Err(e) = evaluation {
                    log::error!("error evaluating task: {}", e);
                } else {
                    let evaluation = evaluation.unwrap();
                    if evaluation.completed {
                        self.state
                            .lock()
                            .await
                            .on_complete(false, Some("evaluation success".to_string()))
                            .await?;
                    } else if let Some(feedback) = evaluation.feedback {
                        self.state.lock().await.add_feedback_to_history(feedback);
                    }
                }
            }
        }

        self.on_event(Event::new(EventType::StateUpdate(state_update)))
            .await?;
        Ok(())
    }

    // TODO: move these feedback strings to a common place

    async fn on_empty_response(&self) {
        let mut mut_state = self.state.lock().await;
        mut_state.metrics.errors.empty_responses += 1;
        mut_state
            .add_unparsed_response_to_history("", "Do not return an empty responses.".to_string());

        self.on_event(Event::new(EventType::EmptyResponse))
            .await
            .unwrap();
    }

    async fn on_chat_response(&self, response: &str) {
        let mut mut_state = self.state.lock().await;
        mut_state.metrics.errors.unparsed_responses += 1;
        mut_state.add_unparsed_response_to_history(
        response,
        "I could not parse any valid tool calls from your response, please correct it according to the instructions.".to_string(),
    );
        self.on_event(Event::new(EventType::TextResponse(response.to_string())))
            .await
            .unwrap();
    }

    async fn on_valid_response(&self) {
        self.state.lock().await.metrics.valid_responses += 1;
    }

    async fn on_invalid_tool_call(&self, tool_call: ToolCall, error: Option<String>) {
        if self.config.cot_tags.contains(&tool_call.tool_name) {
            self.on_event(Event::new(EventType::Thinking(tool_call.argument.unwrap())))
                .await
                .unwrap();
            return;
        }

        let mut mut_state = self.state.lock().await;
        mut_state.metrics.errors.unknown_tool_calls += 1;
        // tell the model that the tool name is wrong
        let name = tool_call.tool_name.clone();

        mut_state.add_error_to_history(
            tool_call.clone(),
            error
                .clone()
                .unwrap_or(format!("'{name}' is not a valid tool name")),
        );

        self.on_event(Event::new(EventType::InvalidToolCall { tool_call, error }))
            .await
            .unwrap();
    }

    async fn on_valid_tool_call(&self, tool_call: &ToolCall) {
        self.state.lock().await.metrics.valid_tool_calls += 1;

        let tool_call = tool_call.clone();

        self.on_event(Event::new(EventType::BeforeToolCall { tool_call }))
            .await
            .unwrap();
    }

    async fn on_tool_time_out(&self, tool_call: ToolCall, start: &std::time::Instant) {
        let mut mut_state = self.state.lock().await;
        mut_state.metrics.errors.timedout_tool_calls += 1;
        // tell the model about the timeout
        mut_state.add_error_to_history(tool_call.clone(), "tool call timed out".to_string());

        self.events_chan
            .send(Event::new(EventType::ToolCallTimeout {
                tool_call,
                elapsed: start.elapsed(),
            }))
            .unwrap();
    }

    async fn on_executed_tool_call(
        &self,
        tool: &Box<dyn Tool>,
        tool_call: ToolCall,
        ret: Result<Option<ToolOutput>>,
        start: &std::time::Instant,
    ) {
        let mut mut_state = self.state.lock().await;
        let mut error = None;
        let mut result = None;

        if let Err(err) = ret {
            mut_state.metrics.errors.errored_tool_calls += 1;
            // tell the model about the error
            mut_state.add_error_to_history(tool_call.clone(), err.to_string());

            error = Some(err.to_string());
        } else {
            let ret = ret.unwrap();
            mut_state.metrics.success_tool_calls += 1;
            // tell the model about the output
            mut_state.add_success_to_history(tool_call.clone(), ret.clone());

            result = ret;
        }

        self.on_event(Event::new(EventType::AfterToolCall {
            tool_call,
            result,
            error,
            elapsed: start.elapsed(),
            complete_task: tool.complete_task(),
        }))
        .await
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

    pub async fn get_variables(&self) -> HashMap<String, String> {
        self.state.lock().await.get_variables().clone()
    }

    pub async fn get_metrics(&self) -> state::metrics::Metrics {
        self.state.lock().await.metrics.clone()
    }

    async fn prepare_step(&mut self) -> Result<ChatOptions> {
        let mut mut_state = self.state.lock().await;

        mut_state.on_step()?;

        self.on_event(Event::new(EventType::MetricsUpdate(
            mut_state.metrics.clone(),
        )))
        .await?;

        let system_prompt = self.config.serializer.system_prompt_for_state(&mut_state)?;
        let prompt = mut_state.to_prompt()?;

        let (system_prompt, prompt) = if self.config.user_only {
            // combine with user prompt for models like the openai/o1 family
            (None, format!("{system_prompt}\n\n{prompt}"))
        } else {
            (Some(system_prompt), prompt)
        };

        let history = mut_state.to_chat_history(&self.config.serializer)?;
        let options = ChatOptions::new(
            system_prompt,
            prompt,
            history,
            self.config.conversation_window,
        );

        Ok(options)
    }

    pub async fn on_event(&self, event: Event) -> Result<()> {
        self.events_chan.send(event)?;
        Ok(())
    }

    pub async fn on_event_type(&self, event_type: EventType) -> Result<()> {
        self.on_event(Event::new(event_type)).await?;
        Ok(())
    }

    pub async fn step(&mut self) -> Result<()> {
        let options = self.prepare_step().await?;

        self.on_state_update(&options, false).await?;

        // wait before inference
        self.config.remote.wait_if_paused().await;

        // run model inference
        let response = self.generator.chat(self.state.clone(), &options).await?;

        // update tokens usage
        self.on_completion(&response).await;

        // parse the model response into tool calls
        let tool_calls = if self.use_native_tools_format && response.tool_calls.is_empty() {
            // no tool calls, attempt to parse the content anyway
            self.config
                .serializer
                .try_parse(response.content.trim())
                .unwrap_or_default()
        } else if !self.use_native_tools_format {
            // use our own parsing strategy
            self.config.serializer.try_parse(response.content.trim())?
        } else {
            response.tool_calls
        };

        // nothing parsed, report the problem to the model
        if tool_calls.is_empty() {
            if response.content.is_empty() {
                self.on_empty_response().await;
            } else {
                self.on_chat_response(&response.content).await;
            }
        } else {
            self.on_valid_response().await;
        }

        // wait before executing tool calls
        self.config.remote.wait_if_paused().await;

        let mut any_state_updates = false;

        // for each parsed call
        for mut call in tool_calls {
            // lookup tool
            let tool = self.state.lock().await.get_tool_by_name(&call.tool_name);
            if tool.is_none() {
                self.on_invalid_tool_call(call.clone(), None).await;
            } else {
                // validate prerequisites
                let tool = tool.unwrap();
                if let Err(err) = self.validate(&mut call, &tool) {
                    self.on_invalid_tool_call(call.clone(), Some(err.to_string()))
                        .await;
                } else {
                    self.on_valid_tool_call(&call).await;

                    // determine if we have a timeout
                    let timeout = if let Some(tool_call_timeout) = tool.timeout().as_ref() {
                        *tool_call_timeout
                    } else if let Some(task_timeout) = self.task_specs.timeout.as_ref() {
                        *task_timeout
                    } else {
                        // one month by default :D
                        Duration::from_secs(60 * 60 * 24 * 30)
                    };

                    let mut execute = true;

                    if tool.requires_user_confirmation() {
                        log::warn!("user confirmation required");

                        let start = std::time::Instant::now();
                        let mut inp = "nope".to_string();
                        while !inp.is_empty() && inp != "n" && inp != "y" {
                            inp = get_user_input(&format!(
                                "{} [Yn] ",
                                call.as_function_call_string()
                            ))
                            .to_ascii_lowercase();
                        }

                        if inp == "n" {
                            log::warn!("tool call rejected by user");
                            self.on_executed_tool_call(
                                &tool,
                                call.clone(),
                                Err(anyhow!("rejected by user".to_owned())),
                                &start,
                            )
                            .await;

                            execute = false;
                        }
                    }

                    if execute {
                        // wait before executing the tool call
                        self.config.remote.wait_if_paused().await;

                        // execute with timeout
                        let start = std::time::Instant::now();
                        let ret = tokio::time::timeout(
                            timeout,
                            tool.run(
                                self.state.clone(),
                                call.named_arguments.to_owned(),
                                call.argument.to_owned(),
                            ),
                        )
                        .await;

                        if ret.is_err() {
                            self.on_tool_time_out(call, &start).await;
                        } else {
                            self.on_executed_tool_call(&tool, call, ret.unwrap(), &start)
                                .await;
                        }

                        // wait after executing the tool call
                        self.config.remote.wait_if_paused().await;

                        if tool.complete_task() {
                            log::debug!("! task complete");
                            self.state.lock().await.on_complete(false, None).await?;
                        }
                    }
                }
            }

            self.on_state_update(&options, true).await?;
            any_state_updates = true;

            // break the loop if we're done
            if self.state.lock().await.is_complete() {
                break;
            }
        }

        // trigger a final state update if there were no state changes
        if !any_state_updates {
            self.on_state_update(&options, true).await?;
        }

        // wait after the step
        self.config.remote.wait_if_paused().await;

        Ok(())
    }

    pub async fn on_end(&mut self) -> Result<()> {
        // report final metrics on exit
        let last_metrics = self.get_metrics().await;

        self.on_event(Event::new(EventType::MetricsUpdate(last_metrics)))
            .await?;
        Ok(())
    }
}
