use std::hash::Hash;

use super::{Message, Request, ResponseFormat, StopEnum, Tool, ToolChoiceEnum};
use serde_json::Value;

/// Provides fluent api for building the request object for chat completion
///
/// The field types, defaults and description could be found from [the official doc](https://console.groq.com/docs/api-reference#chat-create)
///
/// Here and [Request](../../request/struct.Request.html) just a 1:1 mapping from it
#[derive(Debug)]
pub struct RequestBuilder {
    // unused for openai integration only
    logit_bias: Option<serde_json::Value>,
    // unused for openai integration only
    logprobs: bool,         // default false
    frequency_penalty: f32, // defaults to 0
    max_tokens: Option<u32>,
    messages: Vec<Message>,
    model: String,
    n: u32,                          // defaults to 1
    presence_penalty: f32,           // defaults to 0
    response_format: ResponseFormat, // defaults to text,
    seed: Option<i32>,
    stop: Option<StopEnum>,
    stream: bool,     // default false
    temperature: f32, // defaults to 1
    tool_choice: Option<ToolChoiceEnum>,
    tools: Option<Vec<Tool>>,
    top_logprobs: Option<u8>,
    top_p: f32, // defaults to 1
    user: Option<String>,
}

impl Hash for RequestBuilder {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.logprobs.hash(state);
        ((self.frequency_penalty) as i32).hash(state);
        self.max_tokens.hash(state);
        self.messages.hash(state);
        self.model.hash(state);
        self.n.hash(state);
        ((self.presence_penalty) as i32).hash(state);
        self.response_format.hash(state);
        self.seed.hash(state);
        self.stop.hash(state);
        self.stream.hash(state);
        ((self.temperature) as i32).hash(state);
        self.tool_choice.hash(state);
        self.tools.hash(state);
        self.top_logprobs.hash(state);
        ((self.top_p) as i32).hash(state);
        self.user.hash(state);
    }
}

#[derive(Debug, PartialEq)]
pub struct BuilderConfig {
    model: String,
    logit_bias: Option<serde_json::Value>,
    logprobs: Option<bool>,
    frequency_penalty: Option<f32>,
    max_tokens: Option<u32>,
    n: Option<u32>,
    presence_penalty: Option<f32>,
    response_format: Option<ResponseFormat>,
    seed: Option<i32>,
    stop: Option<StopEnum>,
    stream: Option<bool>,
    temperature: Option<f32>,
    tool_choice: Option<ToolChoiceEnum>,
    tools: Option<Vec<Tool>>,
    top_logprobs: Option<u8>,
    top_p: Option<f32>,
    user: Option<String>,
}
impl Hash for BuilderConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.model.hash(state);
        self.logprobs.unwrap_or(false).hash(state);
        (self.frequency_penalty.unwrap_or(0.0) as i32).hash(state);
        self.max_tokens.hash(state);
        self.n.hash(state);
        (self.presence_penalty.unwrap_or(0.0) as i32).hash(state);
        self.response_format.hash(state);
        self.seed.hash(state);
        self.stop.hash(state);
        self.stream.hash(state);
        (self.temperature.unwrap_or(1.0) as i32).hash(state);
        self.tool_choice.hash(state);
        self.tools.hash(state);
        self.top_logprobs.hash(state);
        (self.top_p.unwrap_or(1.0) as i32).hash(state);
        self.user.hash(state);
    }
}

impl RequestBuilder {
    pub fn with_config(cfg: &BuilderConfig) -> Self {
        let mut builder_instance = Self::new(cfg.model.clone());

        if let Some(lg_bias) = cfg.logit_bias.clone() {
            builder_instance = builder_instance.with_logit_bias(lg_bias);
        }
        if let Some(log_probs) = cfg.logprobs {
            builder_instance = builder_instance.with_logprobs(log_probs);
        }
        if let Some(freq_pen) = cfg.frequency_penalty {
            builder_instance = builder_instance.with_frequency_penalty(freq_pen);
        }
        if let Some(max_tok) = cfg.max_tokens {
            builder_instance = builder_instance.with_max_tokens(max_tok);
        }

        if let Some(n) = cfg.n {
            builder_instance = builder_instance.with_n(n);
        }

        if let Some(presence_pen) = cfg.presence_penalty {
            builder_instance = builder_instance.with_presence_penalty(presence_pen);
        }
        if let Some(response_fmt) = cfg.response_format.clone() {
            builder_instance = builder_instance.with_response_fmt(response_fmt);
        }
        if let Some(sed) = cfg.seed {
            builder_instance = builder_instance.with_seed(sed);
        }

        if let Some(stop) = cfg.stop.clone() {
            builder_instance = match stop {
                StopEnum::Token(stp) => builder_instance.with_stop(&stp),
                StopEnum::Tokens(stps) => builder_instance.with_stops(stps),
            }
        }

        if let Some(stream) = cfg.stream {
            builder_instance = builder_instance.with_stream(stream);
        }
        if let Some(temp) = cfg.temperature {
            builder_instance = builder_instance.with_temperature(temp);
        }

        if let Some(tool_choice) = cfg.tool_choice.clone() {
            builder_instance = match tool_choice {
                ToolChoiceEnum::Str(tool_str) => {
                    builder_instance.with_tool_choice_string(tool_str).unwrap()
                }
                ToolChoiceEnum::Tool(tool_inst) => builder_instance.with_tool_choice(tool_inst),
            }
        }

        if let Some(tools) = cfg.tools.clone() {
            builder_instance = builder_instance.with_tools(tools);
        }
        if let Some(top_logprobs) = cfg.top_logprobs {
            builder_instance = builder_instance.with_top_logprobs(top_logprobs);
        }
        if let Some(top_p) = cfg.top_p {
            builder_instance = builder_instance.with_top_p(top_p);
        }
        if let Some(user) = cfg.user.clone() {
            builder_instance = builder_instance.with_user(&user);
        }
        builder_instance
    }

    pub fn get_config(&self) -> BuilderConfig {
        BuilderConfig {
            model: self.model.clone(),
            logit_bias: self.logit_bias.clone(),
            logprobs: Some(self.logprobs),
            frequency_penalty: Some(self.frequency_penalty),
            max_tokens: self.max_tokens,
            n: Some(self.n),
            presence_penalty: Some(self.presence_penalty),
            response_format: Some(self.response_format.clone()),
            seed: self.seed,
            stop: self.stop.clone(),
            stream: Some(self.stream),
            temperature: Some(self.temperature),
            tool_choice: self.tool_choice.clone(),
            tools: self.tools.clone(),
            top_logprobs: self.top_logprobs,
            top_p: Some(self.top_p),
            user: self.user.clone(),
        }
    }

    pub fn from_builder(source: &RequestBuilder) -> Self {
        //! 1 to 1 copy of another RequestBuilder
        let mut builder = Self::with_config(&source.get_config());
        builder.messages.extend(source.messages.clone());
        builder
    }

    pub fn new(model: String) -> Self {
        //! # Important Note
        //! The builder method of modifying messages filed is hidden because the reposibility is
        //! shifted to the client struct.
        //! such that the client struct can maintain the message history and can be reused.
        //!
        //! # Description
        //! Instantiates a RequestBuilder struct with a set of default values for the request object of groq chat completion API.
        //! ```ignore no_run
        //! Self {
        //!    logit_bias: None,
        //!    logprobs: false,
        //!    frequency_penalty: 0.0,
        //!    max_tokens: None,
        //!    messages: Vec::new(),
        //!    model : "no default model".to_string(),
        //!    n: 1,
        //!    presence_penalty: 0.0,
        //!    response_format: ResponseFormat {
        //!        response_type: "text".into(),
        //!    },
        //!    seed: None,
        //!    stop: None,
        //!    stream: false,
        //!    temperature: 1.0,
        //!    tool_choice: None,
        //!    tools: None,
        //!    top_logprobs: None,
        //!    top_p: 1.0,
        //!    user: None,
        //!}
        //!```
        Self {
            logit_bias: None,
            logprobs: false,
            frequency_penalty: 0.0,
            max_tokens: None,
            messages: Vec::new(),
            model,
            n: 1,
            presence_penalty: 0.0,
            response_format: ResponseFormat {
                response_type: "text".into(),
            },
            seed: None,
            stop: None,
            stream: false,
            temperature: 1.0,
            tool_choice: None,
            tools: None,
            top_logprobs: None,
            top_p: 1.0,
            user: None,
        }
    }

    pub fn build(self) -> Request {
        Request {
            logit_bias: self.logit_bias,
            logprobs: self.logprobs,
            frequency_penalty: self.frequency_penalty,
            max_tokens: self.max_tokens,
            messages: self.messages,
            model: self.model,
            n: self.n,
            presence_penalty: self.presence_penalty,
            response_format: self.response_format,
            seed: self.seed,
            stop: self.stop,
            stream: self.stream,
            temperature: self.temperature,
            tool_choice: self.tool_choice,
            tools: self.tools,
            top_logprobs: self.top_logprobs,
            top_p: self.top_p,
            user: self.user,
        }
    }

    pub fn with_logit_bias(mut self, logit_bias: Value) -> Self {
        self.logit_bias = Some(logit_bias);
        self
    }

    pub fn with_logprobs(mut self, logprobs: bool) -> Self {
        self.logprobs = logprobs;
        self
    }

    pub fn with_frequency_penalty(mut self, penalty: f32) -> Self {
        self.frequency_penalty = penalty;
        self
    }

    pub fn with_max_tokens(mut self, n: u32) -> Self {
        self.max_tokens = Some(n);
        self
    }

    pub(crate) fn with_messages(mut self, msgs: Vec<Message>) -> anyhow::Result<Self> {
        anyhow::ensure!(msgs.len() > 0, "message cannot be empty");
        self.messages = msgs;
        Ok(self)
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.into();
        self
    }

    pub fn with_n(mut self, n: u32) -> Self {
        self.n = n;
        self
    }

    pub fn with_presence_penalty(mut self, penalty: f32) -> Self {
        self.presence_penalty = penalty;
        self
    }

    pub fn with_response_fmt(mut self, fmt: ResponseFormat) -> Self {
        self.response_format = fmt;
        self
    }

    pub fn with_seed(mut self, seed: i32) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn with_stop(mut self, stop: &str) -> Self {
        self.stop = Some(StopEnum::Token(stop.into()));
        self
    }

    pub fn with_stops(mut self, stops: Vec<String>) -> Self {
        self.stop = Some(StopEnum::Tokens(stops));
        self
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }

    pub fn with_tool_choice(mut self, tool: Tool) -> Self {
        self.tool_choice = Some(ToolChoiceEnum::Tool(tool));
        self
    }
    pub fn with_auto_tool_choice(mut self) -> Self {
        self.tool_choice = Some(ToolChoiceEnum::Str("auto".into()));
        self
    }

    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn with_top_logprobs(mut self, prob: u8) -> Self {
        self.top_logprobs = Some(prob);
        self
    }

    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = top_p;
        self
    }

    pub fn with_user(mut self, user: &str) -> Self {
        self.user = Some(user.into());
        self
    }

    pub fn is_stream(&self) -> bool {
        //! Check the request object is set to use stream for the completion response or not
        //! - true if the stream flag is on
        //! - false if the stream flag is off
        self.stream
    }

    pub fn with_tool_choice_string(mut self, tool: String) -> anyhow::Result<Self> {
        anyhow::ensure!(
            tool == "auto" || tool == "none",
            "Tool choice of string only allows 'none' or 'auto'"
        );

        self.tool_choice = Some(ToolChoiceEnum::Str(tool));
        Ok(self)
    }
}

#[cfg(test)]
mod builder_test {
    use std::hash::{DefaultHasher, Hash, Hasher};

    use super::{BuilderConfig, RequestBuilder};

    #[test]
    fn can_return_init_config_and_cfg_hash_should_equal() -> anyhow::Result<()> {
        let mut hasher = DefaultHasher::new();
        let mut hasher1 = DefaultHasher::new();
        let cfg = BuilderConfig {
            model: "test".to_string(),
            logit_bias: None,
            logprobs: None,
            frequency_penalty: None,
            max_tokens: None,
            n: None,
            presence_penalty: None,
            response_format: None,
            seed: None,
            stop: None,
            stream: None,
            temperature: None,
            tool_choice: None,
            tools: None,
            top_logprobs: None,
            top_p: None,
            user: None,
        };

        let builder = RequestBuilder::with_config(&cfg);
        let builder1 = RequestBuilder::new("test".into());

        builder.get_config().hash(&mut hasher);
        builder1.get_config().hash(&mut hasher1);

        let builder_hash = hasher.finish();
        let builder1_hash = hasher1.finish();
        assert_eq!(builder_hash, builder1_hash);
        Ok(())
    }

    #[test]
    fn copied_builder_should_have_eq_hash() -> anyhow::Result<()> {
        let mut hasher = DefaultHasher::new();
        let mut hasher1 = DefaultHasher::new();

        let builder = RequestBuilder::new("test".to_string());
        let builder1 = RequestBuilder::from_builder(&builder);

        builder.hash(&mut hasher);
        builder1.hash(&mut hasher1);

        let builder_hash = hasher.finish();
        let builder1_hash = hasher1.finish();
        println!("{}\t{}", builder_hash, builder1_hash);
        assert_eq!(hasher.finish(), hasher1.finish());
        Ok(())
    }
}
