use anyhow::Result;

use super::actions::Namespace;

pub(crate) mod tasklet;

// TODO: comment the shit out of everything.

pub trait Task: std::fmt::Debug {
    fn to_system_prompt(&self) -> Result<String>;
    fn to_prompt(&self) -> Result<String>;
    fn get_functions(&self) -> Vec<Namespace>;

    fn max_history_visibility(&self) -> u16 {
        15
    }

    fn guidance(&self) -> Result<Vec<String>> {
        self.base_guidance()
    }

    fn namespaces(&self) -> Option<Vec<String>> {
        None
    }

    fn base_guidance(&self) -> Result<Vec<String>> {
        // basic rules to extend
        Ok(include_str!("basic_guidance.txt")
            .split('\n')
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect())
    }
}
