use std::collections::HashMap;

use anyhow::Result;
use colored::Colorize;

use super::{Action, Namespace};
use crate::agent::state::State;

#[derive(Debug, Default)]
struct ReadFile {}

impl Action for ReadFile {
    fn name(&self) -> &str {
        "read-file"
    }

    fn description(&self) -> &str {
        include_str!("read_file.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("/path/to/file/to/read")
    }

    fn run(
        &self,
        _state: &State,
        _attributes: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        if let Some(filepath) = payload {
            let ret = std::fs::read_to_string(&filepath);
            if let Ok(contents) = ret {
                println!(
                    "<{}> {} -> {} bytes",
                    self.name().bold(),
                    filepath.yellow(),
                    contents.len()
                );
                return Ok(Some(contents));
            } else {
                let err = ret.err().unwrap();
                println!(
                    "<{}> {} -> {:?}",
                    self.name().bold(),
                    filepath.yellow(),
                    &err
                );

                return Err(anyhow!(err));
            }
        }

        // TODO: check for mandatory payload and attributes while parsing
        Err(anyhow!("no content specified for read-file"))
    }
}

pub(crate) fn get_namespace() -> Namespace {
    Namespace::new_non_default(
        "Filesystem".to_string(),
        include_str!("ns.prompt").to_string(),
        vec![Box::<ReadFile>::default()],
        None,
    )
}
