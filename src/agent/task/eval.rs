use anyhow::Result;
use serde::Deserialize;

use crate::agent::events::StateUpdate;

const SUCCESS_CODE: i32 = 42;

pub struct Evaluation {
    pub completed: bool,
    pub feedback: Option<String>,
}

#[derive(Default, Deserialize, Debug, Clone)]
pub struct Evaluator {
    command: Vec<String>,
}

impl Evaluator {
    pub async fn evaluate(
        &self,
        state: &StateUpdate,
        working_directory: &Option<String>,
    ) -> Result<Evaluation> {
        log::info!("📊 running evaluation ...");

        let mut eval = Evaluation {
            completed: false,
            feedback: None,
        };

        let json = serde_json::to_string(&state)?;

        let mut cmd = tokio::process::Command::new(&self.command[0]);
        if self.command.len() > 1 {
            cmd.args(&self.command[1..]);
        }

        if let Some(working_directory) = working_directory {
            cmd.current_dir(working_directory);
        }

        let mut child = cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        // write JSON to stdin
        if let Some(mut stdin) = child.stdin.take() {
            tokio::io::AsyncWriteExt::write_all(&mut stdin, json.as_bytes()).await?;
        }

        let output = child.wait_with_output().await?;
        if !output.stdout.is_empty() {
            eval.feedback = Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            log::info!("📊 feedback: {}", eval.feedback.as_ref().unwrap());
        }

        if !output.stderr.is_empty() {
            log::error!("📊 {}", String::from_utf8_lossy(&output.stderr));
        }

        eval.completed = output.status.code() == Some(SUCCESS_CODE);

        Ok(eval)
    }
}
