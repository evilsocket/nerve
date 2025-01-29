use std::time::Duration;

use anyhow::Result;
use eval::Evaluator;

use super::namespaces::Namespace;

pub mod eval;
pub mod robopages;
pub mod tasklet;
pub mod variables;

pub trait Task: std::fmt::Debug + Send + Sync {
    fn to_system_prompt(&self) -> Result<String>;
    fn to_prompt(&self) -> Result<String>;
    fn get_functions(&self) -> Vec<Namespace>;

    fn get_working_directory(&self) -> Option<String> {
        None
    }

    fn get_evaluator(&self) -> Option<Evaluator> {
        None
    }

    fn get_timeout(&self) -> Option<Duration> {
        None
    }

    fn get_rag_config(&self) -> Option<mini_rag::Configuration> {
        None
    }

    fn guidance(&self) -> Result<Vec<String>> {
        self.base_guidance()
    }

    fn namespaces(&self) -> Option<Vec<String>> {
        None
    }

    fn base_guidance(&self) -> Result<Vec<String>> {
        // basic rules to extend
        Ok(include_str!("basic_guidance.prompt")
            .split('\n')
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect())
    }
}
