use std::time::Duration;

use async_trait::async_trait;
use colored::Colorize;
use groq_api_rs::completion::{client::Groq, request::builder, response::ErrorResponse};
use lazy_static::lazy_static;
use regex::Regex;

use crate::agent::generator::Message;

use super::{Client, Options};

lazy_static! {
    static ref RETRY_TIME_PARSER: Regex = Regex::new(r"(?m)^.+try again in (.+)s\..*").unwrap();
}

pub struct GroqClient {
    model: String,
    api_key: String,
}

#[async_trait]
impl Client for GroqClient {
    fn new(_: &str, _: u16, model_name: &str, _: u32) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let api_key = std::env::var("GROQ_API_KEY")
            .map_err(|_| anyhow!("Missing GROQ_API_KEY".to_string()))?;

        let model = model_name.to_string();

        Ok(Self { model, api_key })
    }

    async fn chat(&self, options: &Options) -> anyhow::Result<String> {
        let mut chat_history = vec![
            groq_api_rs::completion::message::Message::SystemMessage {
                role: Some("system".to_string()),
                content: Some(options.system_prompt.trim().to_string()),
                name: None,
                tool_call_id: None,
            },
            groq_api_rs::completion::message::Message::UserMessage {
                role: Some("user".to_string()),
                content: Some(options.prompt.trim().to_string()),
                name: None,
                tool_call_id: None,
            },
        ];

        let mut call_idx = 0;

        for m in &options.history {
            chat_history.push(match m {
                Message::Agent(data, invocation) => {
                    let mut tool_call_id = None;
                    if let Some(inv) = invocation {
                        tool_call_id = Some(format!("{}-{}", inv.action, call_idx));
                        call_idx += 1;
                    }

                    groq_api_rs::completion::message::Message::AssistantMessage {
                        role: Some("assistant".to_string()),
                        content: Some(data.trim().to_string()),
                        name: None,
                        tool_call_id,
                        tool_calls: None,
                    }
                }
                Message::Feedback(data, invocation) => {
                    let mut tool_call_id: Option<String> = None;
                    if let Some(inv) = invocation {
                        tool_call_id = Some(format!("{}-{}", inv.action, call_idx));
                    }
                    if tool_call_id.is_some() {
                        groq_api_rs::completion::message::Message::ToolMessage {
                            role: Some("tool".to_string()),
                            content: Some(data.trim().to_string()),
                            name: None,
                            tool_call_id,
                        }
                    } else {
                        groq_api_rs::completion::message::Message::UserMessage {
                            role: Some("user".to_string()),
                            content: Some(data.trim().to_string()),
                            name: None,
                            tool_call_id,
                        }
                    }
                }
            });
        }

        let request = builder::RequestBuilder::new(self.model.clone()).with_stream(false);

        let client = Groq::new(&self.api_key);
        let client = client.add_messages(chat_history);

        let resp = client.create(request).await;
        if let Err(error) = resp {
            if let Some(err_resp) = error.downcast_ref::<ErrorResponse>() {
                // if rate limit exceeded, parse the retry time and retry
                if err_resp.code == 429 {
                    if let Some(caps) = RETRY_TIME_PARSER
                        .captures_iter(&err_resp.error.message)
                        .next()
                    {
                        if caps.len() == 2 {
                            let mut retry_time_str = "".to_string();

                            caps.get(1)
                                .unwrap()
                                .as_str()
                                .clone_into(&mut retry_time_str);

                            if let Ok(retry_time) = retry_time_str.parse::<f32>() {
                                println!(
                                    "{}: rate limit reached for this model, retrying in {}s ...\n",
                                    "WARNING".bold().yellow(),
                                    retry_time,
                                );

                                tokio::time::sleep(Duration::from_millis(
                                    ((retry_time + 1.0) * 1000.0) as u64,
                                ))
                                .await;

                                return self.chat(options).await;
                            }
                        }
                    }

                    eprintln!(
                        "{}: can't parse retry time from error response: {:?}",
                        "WARNING".bold().yellow(),
                        &err_resp
                    );
                }
            }

            return Err(error);
        }

        let choice = match resp.unwrap() {
            groq_api_rs::completion::client::CompletionOption::NonStream(resp) => {
                resp.choices.first().unwrap().to_owned()
            }
            groq_api_rs::completion::client::CompletionOption::Stream(_) => {
                return Err(anyhow!("Groq streaming is not supported yet, if this happens please open an issue on GitHub"));
            }
        };

        Ok(choice.message.content.to_string())
    }
}
