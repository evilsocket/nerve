use std::{
    ops::Deref,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};

use anyhow::Result;
use colored::Colorize;

use super::{
    actions::{self, Group},
    history::{Execution, History},
    memory::{Memories, Memory},
    task::Task,
    Invocation,
};

#[derive(Debug)]
pub struct State {
    task: Box<dyn Task>,
    prev_goal: Mutex<Option<String>>,
    curr_goal: Mutex<String>,
    step: usize,
    max_iterations: usize,

    // model memories
    memories: Mutex<Memories>,

    // available actions and execution history
    action_groups: Vec<Group>,
    action_history: Mutex<History>,

    complete: AtomicBool,
}

impl State {
    pub fn new(task: Box<dyn Task>, max_iterations: usize) -> Result<Self> {
        let complete = AtomicBool::new(false);
        let memories = Mutex::new(Memories::new());
        let action_history = Mutex::new(History::new());

        // add core actions
        let mut action_groups = vec![actions::memory::get_functions()];

        // if the agent can mark as complete the task
        if task.agent_can_complete_autonomously() {
            action_groups.push(actions::task::get_functions());
        }

        // add task specific actions
        action_groups.append(&mut task.get_functions());

        let prev_goal = Mutex::new(None);
        let curr_goal = Mutex::new(task.to_prompt()?);

        let step = 0;

        Ok(Self {
            task,
            memories,
            action_history,
            action_groups,
            complete,
            prev_goal,
            curr_goal,
            max_iterations,
            step,
        })
    }

    pub fn set_current_iteration(&mut self, num: usize) -> Result<()> {
        self.step = num;
        if self.max_iterations > 0 && self.step >= self.max_iterations {
            Err(anyhow!("maximum number of iterations reached"))
        } else {
            Ok(())
        }
    }

    pub fn add_memory(&self, key: String, data: String) {
        println!("\n{}: {}\n", key.bold(), &data.yellow());

        if let Ok(mut guard) = self.memories.lock() {
            guard.insert(key, Memory::new(data));
        }
    }

    pub fn remove_memory(&self, key: &str) -> Option<Memory> {
        if let Ok(mut guard) = self.memories.lock() {
            println!("\n{} clear\n", key.bold());
            return guard.remove(key);
        }
        None
    }

    pub fn set_new_goal(&self, goal: String) {
        println!("{}: '{}'", "goal".bold(), goal.yellow());

        if let Ok(mut curr_g) = self.curr_goal.lock() {
            let prev = curr_g.to_string();

            curr_g.clone_from(&goal);

            if let Ok(mut prev_g) = self.prev_goal.lock() {
                prev_g.replace(prev);
            } else {
                println!("FAILED to acquire prev lock");
            }
        } else {
            println!("FAILED to acquire curr lock");
        }
    }

    pub(crate) fn available_actions_to_string(&self) -> Result<String> {
        let mut md = "".to_string();

        for group in &self.action_groups {
            md += &format!("## {}\n\n", group.name);
            if !group.description.is_empty() {
                md += &format!("{}\n\n", group.description);
            }
            for action in &group.actions {
                md += &format!(
                    "{}\n{}\n\n",
                    action.description(),
                    action.structured_example()
                );
            }
        }

        Ok(md)
    }

    pub fn to_pretty_string(&self) -> Result<String> {
        let current_goal = self.curr_goal.lock().unwrap().to_string();
        let iterations = if self.max_iterations > 0 {
            format!(
                "You are currently at step {} of a maximum of {}.\n",
                self.step + 1,
                self.max_iterations
            )
        } else {
            "".to_string()
        };
        let memories = self.memories.lock().unwrap().to_structured_string()?;
        let last_actions = self
            .action_history
            .lock()
            .unwrap()
            .to_structured_string(self.task.max_history_visibility())?;

        Ok(format!(
            "GOAL: {current_goal}\n{iterations}\n{last_actions}\n{memories}"
        ))
    }

    pub fn to_system_prompt(&self) -> Result<String> {
        let current_goal = self.curr_goal.lock().unwrap().to_string();
        let previous_goal = if let Some(goal) = self.prev_goal.lock().unwrap().deref() {
            format!("Your previous goal was: {goal}")
        } else {
            "".to_string()
        };
        let system_prompt = self.task.to_system_prompt()?;
        let memories = self.memories.lock().unwrap().to_structured_string()?;
        let last_actions = self
            .action_history
            .lock()
            .unwrap()
            .to_structured_string(self.task.max_history_visibility())?;
        let guidance = self
            .task
            .guidance()?
            .into_iter()
            .map(|s| format!("- {}", s))
            .collect::<Vec<String>>()
            .join("\n");
        let available_actions = self.available_actions_to_string()?;

        let iterations = if self.max_iterations > 0 {
            format!(
                "You are currently at step {} of a maximum of {}.",
                self.step + 1,
                self.max_iterations
            )
        } else {
            "".to_string()
        };

        Ok(format!(
            include_str!("state_system_prompt.txt"),
            current_goal = current_goal,
            iterations = iterations,
            previous_goal = previous_goal,
            system_prompt = system_prompt,
            memories = memories,
            available_actions = available_actions,
            last_actions = last_actions,
            guidance = guidance,
        ))
    }

    pub fn to_prompt(&self) -> Result<String> {
        self.task.to_prompt()
    }

    fn add_execution_to_history(
        &self,
        invocation: Invocation,
        result: Option<String>,
        error: Option<String>,
    ) {
        if let Ok(mut guard) = self.action_history.lock() {
            guard.push(Execution::new(invocation, result, error));
        }
    }

    pub async fn execute(&self, invocation: Invocation) -> Result<()> {
        for group in &self.action_groups {
            for action in &group.actions {
                if invocation.action == action.name() {
                    // execute the action
                    let inv = invocation.clone();
                    let ret = action.run(self, invocation.attributes, invocation.payload);

                    if action.add_to_activity() {
                        if let Err(error) = ret {
                            self.add_execution_to_history(inv, None, Some(error.to_string()));
                        } else {
                            self.add_execution_to_history(inv, ret.unwrap(), None);
                        }
                    }

                    return Ok(());
                }
            }
        }

        /*
        Err(anyhow!(
            "action '{}' not available: {:?}",
            &invocation.action,
            &invocation.xml
        ))
         */
        Ok(())
    }

    pub fn on_complete(&self, reason: Option<String>) -> Result<()> {
        println!(
            "\n!!! task-complete: reason:\n\n{}",
            if let Some(r) = reason {
                format!("\n{}", r)
            } else {
                "none".to_string()
            }
        );
        self.complete.store(true, Ordering::SeqCst);
        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        self.complete.load(Ordering::SeqCst)
    }
}
