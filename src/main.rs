#[macro_use]
extern crate anyhow;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use agent::{
    generator,
    task::{self, tasklet::Tasklet, Task},
    Agent,
};

mod agent;
mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::Args::parse();

    let gen_options = args.to_generator_options()?;

    // handle pre defines
    for keyvalue in &args.define {
        let parts: Vec<&str> = keyvalue.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow!("can't parse {keyvalue}, syntax is: key=value"));
        }

        task::tasklet::VAR_CACHE
            .lock()
            .unwrap()
            .insert(parts[0].to_owned(), parts[1].to_owned());
    }

    let mut tasklet: Tasklet = Tasklet::from_path(&args.tasklet)?;

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
