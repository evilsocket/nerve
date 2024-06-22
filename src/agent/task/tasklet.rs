use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::{collections::HashMap, sync::Mutex};

use anyhow::Result;
use colored::Colorize;
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_trim::*;

use super::Task;
use crate::{
    agent::namespaces::{Action, Namespace},
    cli,
};

const STATE_COMPLETE_EXIT_CODE: i32 = 65;

lazy_static! {
    pub(crate) static ref VAR_CACHE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

fn parse_pre_defined_values(defines: &Vec<String>) -> Result<()> {
    for keyvalue in defines {
        let parts: Vec<&str> = keyvalue.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow!("can't parse {keyvalue}, syntax is: key=value"));
        }

        VAR_CACHE
            .lock()
            .unwrap()
            .insert(parts[0].to_owned(), parts[1].to_owned());
    }

    Ok(())
}

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
    #[serde(deserialize_with = "string_trim")]
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
        self.example_payload.as_deref()
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
            let mut var_cache = VAR_CACHE.lock().unwrap();

            // more complex command line
            for part in &parts[1..] {
                if part.as_bytes()[0] == b'$' {
                    // variable, the order of lookup is:
                    //  0. environment variable
                    //  1. cache
                    //  2. if an alternative default was provided via || use it
                    //  3. ask the user (and cache)
                    let var_name = part.trim_start_matches('$');

                    let (var_name, var_default) =
                        if let Some((name, default_value)) = var_name.split_once("||") {
                            (name, Some(default_value))
                        } else {
                            (var_name, None)
                        };

                    if let Ok(value) = std::env::var(var_name) {
                        // get from env
                        cmd.arg(value);
                    } else if let Some(cached) = var_cache.get(var_name) {
                        // get from cached
                        cmd.arg(cached);
                    } else if let Some(var_default) = var_default {
                        // get from default
                        cmd.arg(var_default);
                    } else {
                        // get from user
                        let var_value =
                            cli::get_user_input(&format!("\nplease set ${}: ", var_name.yellow()));

                        var_cache.insert(var_name.to_owned(), var_value.clone());
                        cmd.arg(var_value);
                    }
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
            // println!("# {}", payload.bold());
            cmd.arg(payload);
        }

        println!(
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

        // println!("! {:?}", &cmd);

        let output = cmd.output();
        if let Ok(output) = output {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let out = String::from_utf8_lossy(&output.stdout).trim().to_string();

            if !err.is_empty() {
                println!(
                    "\n{}\n",
                    if err.len() > self.max_shown_output {
                        format!(
                            "{}\n{}",
                            &err[0..self.max_shown_output].red(),
                            "<truncated>".yellow()
                        )
                    } else {
                        err.red().to_string()
                    }
                );
            }

            if !out.is_empty() {
                println!(
                    "\n{}\n",
                    if out.len() > self.max_shown_output {
                        format!(
                            "{}\n{}",
                            &out[0..self.max_shown_output],
                            "<truncated>".yellow()
                        )
                    } else {
                        out.to_string()
                    }
                );
            }

            let exit_code = output.status.code().unwrap_or(0);
            // println!("exit_code={}", exit_code);
            if exit_code == STATE_COMPLETE_EXIT_CODE {
                state.on_complete(false, Some(out))?;
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
    folder: String,
    #[serde(skip_deserializing, skip_serializing)]
    pub name: String,
    #[serde(deserialize_with = "string_trim")]
    system_prompt: String,
    pub prompt: Option<String>,
    using: Option<Vec<String>>,
    guidance: Option<Vec<String>>,
    functions: Option<Vec<FunctionGroup>>,
}

impl Tasklet {
    pub fn from_path(path: &str, defines: &Vec<String>) -> Result<Self> {
        parse_pre_defined_values(defines)?;

        let ppath = PathBuf::from_str(path)?;
        if ppath.is_dir() {
            Self::from_folder(path)
        } else {
            Self::from_yaml_file(path)
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

            // println!("tasklet = {:?}", &tasklet);

            Ok(tasklet)
        }
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
