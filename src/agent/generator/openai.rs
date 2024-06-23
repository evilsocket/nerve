use std::time::Duration;

use colored::Colorize;
use duration_string::DurationString;
use openai_api_rust::chat::*;
use openai_api_rust::*;

use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;

use super::{Client, Message, Options};

lazy_static! {
    static ref RETRY_TIME_PARSER: Regex =
        Regex::new(r"(?m)^.+try again in (.+)\. Visit.*").unwrap();
}

pub struct OpenAIClient {
    model: String,
    client: OpenAI,
}

#[async_trait]
impl Client for OpenAIClient {
    fn new(_: &str, _: u16, model_name: &str, _: u32) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        // Load API key from environment OPENAI_API_KEY.
        // You can also hadcode through `Auth::new(<your_api_key>)`, but it is not recommended.
        let auth = Auth::from_env().map_err(|e| anyhow!(e))?;
        let client = OpenAI::new(auth, "https://api.openai.com/v1/");
        let model = model_name.to_string();

        Ok(Self { model, client })
    }

    async fn chat(&self, options: &Options) -> anyhow::Result<String> {
        let mut chat_history = vec![
            openai_api_rust::Message {
                role: Role::System,
                content: options.system_prompt.trim().to_string(),
            },
            openai_api_rust::Message {
                role: Role::User,
                content: options.prompt.trim().to_string(),
            },
        ];

        for m in &options.history {
            chat_history.push(match m {
                Message::Agent(data, _) => openai_api_rust::Message {
                    role: Role::Assistant,
                    content: data.trim().to_string(),
                },
                Message::Feedback(data, _) => openai_api_rust::Message {
                    role: Role::User,
                    content: data.trim().to_string(),
                },
            });
        }

        let body = ChatBody {
            model: self.model.to_string(),
            max_tokens: None,
            temperature: None,
            top_p: None,
            n: None,
            stream: Some(false),
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            messages: chat_history,
        };
        let resp = self.client.chat_completion_create(&body);

        if let Err(error) = resp {
            // if rate limit exceeded, parse the retry time and retry
            if let Some(caps) = RETRY_TIME_PARSER
                .captures_iter(&format!("{}", &error))
                .next()
            {
                if caps.len() == 2 {
                    let mut retry_time_str = "".to_string();

                    caps.get(1)
                        .unwrap()
                        .as_str()
                        .clone_into(&mut retry_time_str);

                    // DurationString can't handle decimals like Xm3.838383s
                    if retry_time_str.contains('.') {
                        let (val, _) = retry_time_str.split_once('.').unwrap();
                        retry_time_str = format!("{}s", val);
                    }

                    if let Ok(retry_time) = retry_time_str.parse::<DurationString>() {
                        println!(
                            "{}: rate limit reached for this model, retrying in {} ...\n",
                            "WARNING".bold().yellow(),
                            retry_time,
                        );

                        tokio::time::sleep(
                            retry_time.checked_add(Duration::from_millis(1000)).unwrap(),
                        )
                        .await;

                        return self.chat(options).await;
                    } else {
                        eprintln!("can't parse '{}'", &retry_time_str);
                    }
                } else {
                    eprintln!("cap len wrong");
                }
            }

            return Err(anyhow!(error));
        }

        let choice = resp.unwrap().choices;
        let message = &choice[0].message.as_ref().unwrap();

        Ok(message.content.to_string())
    }
}
