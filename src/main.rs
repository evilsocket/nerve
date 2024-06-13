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

    // create generator
    let gen_options = args.to_generator_options()?;
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

    // read and create the tasklet
    let mut tasklet: Tasklet = Tasklet::from_path(&args.tasklet, &args.define)?;
    // if the tasklet doesn't provide a prompt
    if tasklet.prompt.is_none() {
        tasklet.prompt = Some(if let Some(prompt) = &args.prompt {
            // if passed by command line
            prompt.to_string()
        } else {
            // ask the user
            cli::get_user_input("enter task> ")
        });
    }
    let task = Box::new(tasklet);

    println!("{}: {}", "task".bold(), task.to_prompt()?.trim().yellow());

    // create the agent given the generator, task and a set of options
    let mut agent = Agent::new(generator, task, args.to_agent_options())?;

    println!(
        "{}: {}\n",
        "namespaces".bold(),
        agent.state().used_namespaces().join(", ")
    );

    // keep going until the task is complete or a fatal error is reached
    while !agent.get_state().is_complete() {
        // next step
        if let Err(error) = agent.step().await {
            println!("{}", error.to_string().bold().red());
            return Err(error);
        }
    }

    Ok(())
}
