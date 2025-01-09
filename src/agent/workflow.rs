use std::{path::PathBuf, str::FromStr};

use anyhow::Result;
use indexmap::IndexMap;
use serde::Deserialize;

#[derive(Default, Deserialize, Debug, Clone)]
pub struct WorkflowTask {
    pub generator: Option<String>,
}

#[derive(Default, Deserialize, Debug, Clone)]
pub struct Workflow {
    #[serde(skip_deserializing, skip_serializing)]
    pub folder: String,

    pub name: String,
    pub description: Option<String>,
    pub tasks: IndexMap<String, WorkflowTask>,
    pub report: Option<String>,
}

impl Workflow {
    pub fn from_path(workflow_path: &str) -> Result<Self> {
        let mut workflow_path = PathBuf::from_str(workflow_path)?;
        // try to look it up in ~/.nerve/workflows
        if !workflow_path.exists() {
            let in_home = crate::agent::data_path("workflows")?.join(&workflow_path);
            if in_home.exists() {
                workflow_path = in_home;
            }
        }

        if workflow_path.is_dir() {
            Self::from_folder(workflow_path.to_str().unwrap())
        } else {
            Self::from_yaml_file(workflow_path.to_str().unwrap())
        }
    }

    fn from_folder(path: &str) -> Result<Self> {
        let filepath = PathBuf::from_str(path);
        if let Err(err) = filepath {
            Err(anyhow!("could not read {path}: {err}"))
        } else {
            Self::from_yaml_file(filepath.unwrap().join("workflow.yml").to_str().unwrap())
        }
    }

    fn from_yaml_file(filepath: &str) -> Result<Self> {
        let canon = std::fs::canonicalize(filepath);
        if let Err(err) = canon {
            Err(anyhow!("could not read {filepath}: {err}"))
        } else {
            let canon = canon.unwrap();
            let workflow_parent_folder = if let Some(folder) = canon.parent() {
                folder
            } else {
                return Err(anyhow!("can't find parent folder of {}", canon.display()));
            };

            let yaml = std::fs::read_to_string(&canon)?;
            let mut workflow: Self = serde_yaml::from_str(&yaml)?;

            // used to set the working directory while running the task
            workflow.folder = if let Some(folder) = workflow_parent_folder.to_str() {
                folder.to_string()
            } else {
                return Err(anyhow!("can't get string of {:?}", workflow_parent_folder));
            };

            // at least one task is required
            if workflow.tasks.is_empty() {
                return Err(anyhow!("no tasks found in workflow"));
            }

            // check that each task exists in the workflow folder
            for (task_name, _) in &workflow.tasks {
                if !workflow_parent_folder
                    .join(task_name)
                    .with_extension("yml")
                    .exists()
                {
                    return Err(anyhow!("task {} does not exist", task_name));
                }
            }

            log::debug!("workflow = {:?}", &workflow);

            Ok(workflow)
        }
    }
}
