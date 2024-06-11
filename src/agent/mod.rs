use generator::Generator;

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
    pub persist_prompt_path: Option<String>,
    pub persist_state_path: Option<String>,
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

    fn dump_state(&self) -> Result<()> {
        if let Some(state_path) = &self.options.persist_state_path {
            std::fs::write(state_path, self.state.to_pretty_string()?)?;
        }

        if let Some(prompt_path) = &self.options.persist_prompt_path {
            std::fs::write(prompt_path, self.state.to_system_prompt()?)?;
        }

        Ok(())
    }

    pub async fn step(&mut self) -> Result<()> {
        self.state.on_next_iteration()?;

        let system_prompt = self.state.to_system_prompt()?;
        let prompt = self.state.to_prompt()?;

        self.dump_state()?;

        // run model inference
        let response: String = self
            .generator
            .run(
                &system_prompt,
                &prompt,
                self.state.to_chat_history(self.max_history as usize)?,
            )
            .await?;

        // parse the model response into invocations
        let invocations = parse_model_response(&response)?;
        let mut prev: Option<String> = None;

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

            self.dump_state()?;
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
