use std::collections::HashMap;
use std::process::Command;

use anyhow::Result;
use colored::Colorize;
use serde::Deserialize;

use crate::agent::actions::{Action, Group};

use super::Task;

const STATE_COMPLETE_EXIT_CODE: i32 = 65;

#[derive(Default, Deserialize, Debug, Clone)]
pub struct TaskletAction {
    #[serde(skip_deserializing, skip_serializing)]
    working_directory: String,

    name: String,
    description: String,
    args: Option<HashMap<String, String>>,
    example_payload: String,
    tool: String,
}

impl Action for TaskletAction {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn example_payload(&self) -> Option<&str> {
        Some(self.example_payload.as_str())
    }

    fn attributes(&self) -> Option<HashMap<String, String>> {
        self.args.clone()
    }

    fn run(
        &self,
        state: &crate::agent::state::State,
        attributes: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        println!(
            "{}{}{}",
            self.name.bold(),
            if attributes.is_some() {
                format!(
                    " {}",
                    attributes
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|(key, value)| format!("{key}={value}"))
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            } else {
                "".to_string()
            },
            if payload.is_some() {
                format!(" {}", payload.as_ref().unwrap().red())
            } else {
                "".to_string()
            }
        );

        if self.tool.is_empty() {
            return Err(anyhow!("no tool defined"));
        }

        let mut cmd = Command::new(&self.tool);

        cmd.current_dir(&self.working_directory);

        if let Some(attrs) = attributes {
            for (key, value) in attrs {
                cmd.args([&format!("--{}", key), &value]);
            }
        }

        if let Some(payload) = &payload {
            // println!("# {}", payload.bold());
            cmd.arg(payload);
        }

        let output = cmd.output();
        if let Ok(output) = output {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let out = String::from_utf8_lossy(&output.stdout).trim().to_string();

            if !err.is_empty() {
                println!("{}", &err);
            }

            if !out.is_empty() {
                println!("{}", &out);
            }

            let exit_code = output.status.code().unwrap_or(0);
            // println!("exit_code={}", exit_code);
            if exit_code == STATE_COMPLETE_EXIT_CODE {
                state.on_complete(Some("task complete".to_string()))?;
                return Ok(Some("task complete".to_string()));
            }

            if !err.is_empty() {
                Err(anyhow!(err))
            } else if output.status.success() {
                Ok(Some(out))
            } else {
                Err(anyhow!(err))
            }
        } else {
            let err = output.err().unwrap().to_string();
            println!("ERROR: {}", &err);

            Err(anyhow!(err))
        }
    }
}

#[derive(Default, Deserialize, Debug, Clone)]
pub struct FunctionGroup {
    pub name: String,
    pub description: Option<String>,
    pub actions: Vec<TaskletAction>,
}

impl FunctionGroup {
    fn compile(&self, working_directory: &str) -> Result<Group> {
        let mut actions: Vec<Box<dyn Action>> = vec![];
        for tasklet_action in &self.actions {
            let mut action = tasklet_action.clone();
            action.working_directory = working_directory.to_string();
            actions.push(Box::new(action));
        }

        Ok(Group::new(
            self.name.to_string(),
            if let Some(desc) = &self.description {
                desc.to_string()
            } else {
                "".to_string()
            },
            actions,
        ))
    }
}

#[derive(Default, Deserialize, Debug, Clone)]
pub struct Tasklet {
    #[serde(skip_deserializing, skip_serializing)]
    folder: String,
    system_prompt: String,
    pub prompt: Option<String>,
    agent_autonomous: bool,
    guidance: Vec<String>,
    functions: Vec<FunctionGroup>,
}

impl Tasklet {
    pub fn from_yaml_file(filepath: &str) -> Result<Self> {
        let filepath = std::fs::canonicalize(filepath)?;
        let folder = if let Some(folder) = filepath.parent() {
            folder
        } else {
            return Err(anyhow!(
                "can't find parent folder of {}",
                filepath.display()
            ));
        };

        let yaml = std::fs::read_to_string(&filepath)?;
        let mut tasklet: Self = serde_yaml::from_str(&yaml)?;

        tasklet.folder = if let Some(folder) = folder.to_str() {
            folder.to_string()
        } else {
            return Err(anyhow!("can't get string of {:?}", folder));
        };

        // println!("tasklet = {:?}", &tasklet);

        Ok(tasklet)
    }
}

impl Task for Tasklet {
    fn to_system_prompt(&self) -> Result<String> {
        Ok(self.system_prompt.to_string())
    }

    fn to_prompt(&self) -> Result<String> {
        if let Some(prompt) = &self.prompt {
            Ok(prompt.to_string())
        } else {
            Err(anyhow!("prompt not specified"))
        }
    }

    fn agent_can_complete_autonomously(&self) -> bool {
        self.agent_autonomous
    }

    fn guidance(&self) -> Result<Vec<String>> {
        let base = self.base_guidance()?;
        // extend the set of basic rules
        Ok([base, self.guidance.clone()].concat())
    }

    fn get_functions(&self) -> Vec<Group> {
        let mut groups = vec![];

        for group in &self.functions {
            groups.push(group.compile(&self.folder).unwrap());
        }

        groups
    }
}
