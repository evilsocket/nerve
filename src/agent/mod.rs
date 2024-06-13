use colored::Colorize;
use generator::{Generator, GeneratorOptions, ModelOptions};

use anyhow::Result;
use parsing::parse_model_response;
use state::State;
use task::Task;

pub mod actions;
pub mod generator;
mod parsing;
pub mod state;
pub mod task;

#[derive(Debug, Clone)]
pub struct AgentOptions {
    pub max_iterations: usize,
    pub persist_path: Option<String>,
}

pub struct Agent {
    generator: Box<dyn Generator>,
    state: State,
    options: AgentOptions,
    max_history: u16,
}

impl Agent {
    pub fn new(
        generator: Box<dyn Generator>,
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

    fn save_system_prompt_if_needed(&self, system_prompt: Option<&str>) -> Result<()> {
        if let Some(prompt_path) = &self.options.persist_path {
            let data = match system_prompt {
                // regenerate
                None => self.state.to_system_prompt()?,
                // use the provided one
                Some(p) => p.to_string(),
            };

            std::fs::write(prompt_path, data)?;
        }

        Ok(())
    }

    pub async fn step(&mut self) -> Result<()> {
        self.state.on_next_iteration()?;

        let system_prompt = self.state.to_system_prompt()?;
        let prompt = self.state.to_prompt()?;
        let history = self.state.to_chat_history(self.max_history as usize)?;

        self.save_system_prompt_if_needed(Some(&system_prompt))?;

        // run model inference
        let options =
            GeneratorOptions::new(system_prompt, prompt, history, ModelOptions::default());
        let response = self.generator.run(options).await?.trim().to_string();

        // parse the model response into invocations
        let invocations = parse_model_response(&response)?;
        let mut prev: Option<String> = None;

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
            // avoid running the same command twince in a row
            if let Some(p) = prev.as_ref() {
                if inv.xml == *p {
                    println!(".");
                    continue;
                }
            }

            prev = Some(inv.xml.clone());

            // see if valid action and execute
            if let Err(e) = self.state.execute(inv).await {
                println!("ERROR: {}", e);
            }

            self.save_system_prompt_if_needed(None)?;
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
