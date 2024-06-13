#[macro_use]
extern crate anyhow;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use agent::{
    generator,
    task::{tasklet::Tasklet, Task},
    Agent,
};

mod agent;
mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();

    let gen_options = args.to_generator_options()?;

    let mut tasklet: Tasklet = Tasklet::from_path(&args.tasklet, &args.define)?;

    if tasklet.prompt.is_none() {
        tasklet.prompt = Some(if let Some(prompt) = &args.prompt {
            prompt.to_string()
        } else {
            cli::get_user_input("enter task> ")
        });
    }
    let task = Box::new(tasklet);

    println!("{}: {}", "task".bold(), task.to_prompt()?.trim().yellow());

    let generator = generator::factory(
        &gen_options.type_name,
        &gen_options.host,
        gen_options.port,
        &gen_options.model_name,
    )?;

    println!(
        "using {}@{}:{}",
        gen_options.model_name.bold(),
        gen_options.host.dimmed(),
        gen_options.port.to_string().dimmed()
    );

    let mut agent = Agent::new(generator, task, args.to_agent_options())?;

    println!(
        "{}: {}\n",
        "namespaces".bold(),
        agent.state().used_namespaces().join(", ")
    );

    while !agent.get_state().is_complete() {
        if let Err(error) = agent.step().await {
            println!("{}", error.to_string().bold().red());
            return Err(error);
        }
    }

    Ok(())
}
