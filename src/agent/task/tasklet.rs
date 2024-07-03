use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::{collections::HashMap, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use duration_string::DurationString;
use serde::Deserialize;
use serde_trim::*;
use simple_home_dir::home_dir;

use super::{variables::interpolate_variables, Task};
use crate::{
    agent::{
        namespaces::{Action, Namespace},
        state::SharedState,
        task::variables::{parse_pre_defined_values, parse_variable_expr},
    },
    cli,
};

const STATE_COMPLETE_EXIT_CODE: i32 = 65;

fn default_max_shown_output() -> usize {
    256
}

#[derive(Default, Deserialize, Debug, Clone)]
pub struct TaskletAction {
    #[serde(skip_deserializing, skip_serializing)]
    working_directory: String,
    #[serde(default = "default_max_shown_output")]
    max_shown_output: usize,
    #[serde(deserialize_with = "string_trim")]
    name: String,
    #[serde(deserialize_with = "string_trim")]
    description: String,
    args: Option<HashMap<String, String>>,
    example_payload: Option<String>,
    timeout: Option<String>,
    #[serde(deserialize_with = "string_trim")]
    tool: String,
}

#[async_trait]
impl Action for TaskletAction {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn example_payload(&self) -> Option<&str> {
        self.example_payload.as_deref()
    }

    fn example_attributes(&self) -> Option<HashMap<String, String>> {
        self.args.clone()
    }

    fn timeout(&self) -> Option<Duration> {
        if let Some(timeout) = &self.timeout {
            if let Ok(tm) = timeout.parse::<DurationString>() {
                return Some(*tm);
            } else {
                log::error!("can't parse '{}' as duration string", timeout);
            }
        }
        None
    }

    async fn run(
        &self,
        state: SharedState,
        attributes: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        let parts: Vec<String> = self
            .tool
            .split_whitespace()
            .map(|x| x.trim())
            .filter(|x| !x.is_empty())
            .map(|x| x.to_string())
            .collect();

        if parts.is_empty() {
            return Err(anyhow!("no tool defined"));
        }

        let mut cmd = Command::new(&parts[0]);
        if parts.len() > 1 {
            // more complex command line
            for part in &parts[1..] {
                if part.as_bytes()[0] == b'$' {
                    let (_, var_value) = parse_variable_expr(part)?;
                    cmd.arg(var_value);
                } else {
                    // raw value
                    cmd.arg(part);
                }
            }
        }

        cmd.current_dir(&self.working_directory);

        if let Some(attrs) = &attributes {
            for (key, value) in attrs {
                cmd.args([&format!("--{}", key), value]);
            }
        }

        if let Some(payload) = &payload {
            cmd.arg(payload);
        }

        log::info!(
            "{}{}{}",
            self.name.bold(),
            if payload.is_some() {
                format!(" {}", payload.as_ref().unwrap().red())
            } else {
                "".to_string()
            },
            if attributes.is_some() {
                format!(
                    " {}",
                    attributes
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|(key, value)| format!("{key}{}{}", "=".dimmed(), value.bright_blue()))
                        .collect::<Vec<String>>()
                        .join(" ")
                )
            } else {
                "".to_string()
            },
        );

        log::debug!("! {:?}", &cmd);

        let output = cmd.output();
        if let Ok(output) = output {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let out = String::from_utf8_lossy(&output.stdout).trim().to_string();

            if !err.is_empty() {
                log::error!(
                    "{}",
                    if err.len() > self.max_shown_output {
                        format!(
                            "{}\n{}",
                            &err[0..self.max_shown_output].red(),
                            "... truncated ...".yellow()
                        )
                    } else {
                        err.red().to_string()
                    }
                );
            }

            if !out.is_empty() {
                let lines = if out.len() > self.max_shown_output {
                    let end = out
                        .char_indices()
                        .map(|(i, _)| i)
                        .nth(self.max_shown_output)
                        .unwrap();
                    let ascii = &out[0..end];
                    format!("{}\n{}", ascii, "... truncated ...")
                } else {
                    out.to_string()
                }
                .split('\n')
                .map(|s| s.dimmed().to_string())
                .collect::<Vec<String>>();

                for line in lines {
                    log::info!("{}", line);
                }
            }

            let exit_code = output.status.code().unwrap_or(0);
            log::debug!("exit_code={}", exit_code);
            if exit_code == STATE_COMPLETE_EXIT_CODE {
                state.lock().await.on_complete(false, Some(out))?;
                return Ok(Some("task complete".to_string()));
            }

            if !err.is_empty() {
                Err(anyhow!(err))
            } else {
                Ok(Some(out))
            }
        } else {
            let err = output.err().unwrap().to_string();
            log::error!("{}", &err);
            Err(anyhow!(err))
        }
    }
}

#[derive(Default, Deserialize, Debug, Clone)]
struct FunctionGroup {
    #[serde(deserialize_with = "string_trim")]
    pub name: String,
    pub description: Option<String>,
    pub actions: Vec<TaskletAction>,
}

impl FunctionGroup {
    fn compile(&self, working_directory: &str) -> Result<Namespace> {
        let mut actions: Vec<Box<dyn Action>> = vec![];
        for tasklet_action in &self.actions {
            let mut action = tasklet_action.clone();
            action.working_directory = working_directory.to_string();
            actions.push(Box::new(action));
        }

        Ok(Namespace::new_default(
            self.name.to_string(),
            if let Some(desc) = &self.description {
                desc.to_string()
            } else {
                "".to_string()
            },
            actions,
            None, // TODO: let tasklets declare custom storages?
        ))
    }
}

#[derive(Default, Deserialize, Debug, Clone)]
pub struct Tasklet {
    #[serde(skip_deserializing, skip_serializing)]
    pub folder: String,
    #[serde(skip_deserializing, skip_serializing)]
    pub name: String,
    #[serde(deserialize_with = "string_trim")]
    system_prompt: String,
    pub prompt: Option<String>,
    pub rag: Option<mini_rag::Configuration>,
    timeout: Option<String>,
    using: Option<Vec<String>>,
    guidance: Option<Vec<String>>,
    functions: Option<Vec<FunctionGroup>>,
}

impl Tasklet {
    pub fn from_path(path: &str, defines: &Vec<String>) -> Result<Self> {
        parse_pre_defined_values(defines)?;

        let mut ppath = PathBuf::from_str(path)?;

        // try to look it up in ~/.nerve/tasklets
        if !ppath.exists() {
            let in_home = home_dir()
                .unwrap()
                .join(PathBuf::from_str(".nerve/tasklets")?.join(&ppath));
            if in_home.exists() {
                ppath = in_home;
            }
        }

        if ppath.is_dir() {
            Self::from_folder(ppath.to_str().unwrap())
        } else {
            Self::from_yaml_file(ppath.to_str().unwrap())
        }
    }

    fn from_folder(path: &str) -> Result<Self> {
        let filepath = PathBuf::from_str(path);
        if let Err(err) = filepath {
            Err(anyhow!("could not read {path}: {err}"))
        } else {
            Self::from_yaml_file(filepath.unwrap().join("task.yml").to_str().unwrap())
        }
    }

    fn from_yaml_file(filepath: &str) -> Result<Self> {
        let canon = std::fs::canonicalize(filepath);
        if let Err(err) = canon {
            Err(anyhow!("could not read {filepath}: {err}"))
        } else {
            let canon = canon.unwrap();
            let tasklet_parent_folder = if let Some(folder) = canon.parent() {
                folder
            } else {
                return Err(anyhow!("can't find parent folder of {}", canon.display()));
            };

            let yaml = std::fs::read_to_string(&canon)?;
            let mut tasklet: Self = serde_yaml::from_str(&yaml)?;

            tasklet.folder = if let Some(folder) = tasklet_parent_folder.to_str() {
                folder.to_string()
            } else {
                return Err(anyhow!("can't get string of {:?}", tasklet_parent_folder));
            };

            tasklet.name = if canon.ends_with("task.yaml") || canon.ends_with("task.yml") {
                tasklet_parent_folder
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned()
            } else {
                canon.file_stem().unwrap().to_str().unwrap().to_owned()
            };

            log::debug!("tasklet = {:?}", &tasklet);

            Ok(tasklet)
        }
    }

    pub fn prepare(&mut self, user_prompt: &Option<String>) -> Result<()> {
        if self.prompt.is_none() {
            self.prompt = Some(if let Some(prompt) = &user_prompt {
                // if passed by command line
                prompt.to_string()
            } else {
                // ask the user
                cli::get_user_input("enter task> ")
            });
        }

        // parse any variable
        self.prompt = Some(interpolate_variables(self.prompt.as_ref().unwrap().trim())?);

        // fix paths
        if let Some(rag) = self.rag.as_mut() {
            let src_path = PathBuf::from(&rag.source_path);
            if src_path.is_relative() {
                rag.source_path =
                    std::fs::canonicalize(PathBuf::from(&self.folder).join(src_path))?
                        .display()
                        .to_string();
            }

            let data_path = PathBuf::from(&rag.data_path);
            if data_path.is_relative() {
                rag.data_path = std::fs::canonicalize(PathBuf::from(&self.folder).join(data_path))?
                    .display()
                    .to_string();
            }
        }

        Ok(())
    }
}

impl Task for Tasklet {
    fn get_timeout(&self) -> Option<std::time::Duration> {
        if let Some(timeout) = &self.timeout {
            if let Ok(tm) = timeout.parse::<DurationString>() {
                return Some(*tm);
            } else {
                log::error!("can't parse '{}' as duration string", timeout);
            }
        }
        None
    }

    fn get_rag_config(&self) -> Option<mini_rag::Configuration> {
        self.rag.clone()
    }

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

    fn namespaces(&self) -> Option<Vec<String>> {
        self.using.clone()
    }

    fn guidance(&self) -> Result<Vec<String>> {
        let base = self.base_guidance()?;
        // extend the set of basic rules
        Ok([base, self.guidance.as_ref().unwrap_or(&vec![]).clone()].concat())
    }

    fn get_functions(&self) -> Vec<Namespace> {
        let mut groups = vec![];

        if let Some(custom_functions) = self.functions.as_ref() {
            for group in custom_functions {
                groups.push(group.compile(&self.folder).unwrap());
            }
        }

        groups
    }
}
