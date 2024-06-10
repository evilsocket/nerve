use anyhow::Result;

use super::actions::Group;

pub(crate) mod tasklet;

pub trait Task: std::fmt::Debug {
    fn to_system_prompt(&self) -> Result<String>;
    fn to_prompt(&self) -> Result<String>;
    fn get_functions(&self) -> Vec<Group>;

    fn agent_can_complete_autonomously(&self) -> bool {
        true
    }

    fn guidance(&self) -> Result<Vec<String>> {
        self.base_guidance()
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
