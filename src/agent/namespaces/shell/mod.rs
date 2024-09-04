use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use tokio::process::Command;

use crate::agent::state::SharedState;

use super::{Action, Namespace};

#[derive(Debug, Default, Clone)]
struct Shell {}

#[async_trait]
impl Action for Shell {
    fn name(&self) -> &str {
        "shell"
    }

    fn description(&self) -> &str {
        include_str!("shell.prompt")
    }

    fn example_payload(&self) -> Option<&str> {
        Some("ls -la")
    }

    fn requires_user_confirmation(&self) -> bool {
        // this one definitely does
        true
    }

    async fn run(
        &self,
        _: SharedState,
        _: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        let command = payload.unwrap();
        log::debug!("{}", &command);

        // TODO: make the shell configurable
        let output = Command::new("/bin/sh")
            .arg("-c")
            .arg(&command)
            .output()
            .await?;

        let mut result = String::from_utf8_lossy(&output.stdout).to_string();
        println!("{}", &result);

        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !stderr.is_empty() {
            eprintln!("{}", &stderr);
            result += &format!("\nSTDERR: {}\n", stderr);
        }

        let exit_code = output.status.code().unwrap_or(-1);
        if exit_code != 0 {
            result += &format!("\nEXIT CODE: {}", exit_code);
        }

        log::debug!("{}", &result);

        Ok(Some(result))
    }
}

pub(crate) fn get_namespace() -> Namespace {
    Namespace::new_non_default(
        "Shell".to_string(),
        include_str!("ns.prompt").to_string(),
        vec![Box::<Shell>::default()],
        None,
    )
}
