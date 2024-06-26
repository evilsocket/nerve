#[macro_use]
extern crate anyhow;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use agent::{generator, serialization, task::tasklet::Tasklet, Agent};

mod agent;
mod cli;

const APP_NAME: &str = env!("CARGO_BIN_NAME");
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[allow(clippy::type_complexity)]
fn setup_models(
    args: &cli::Args,
) -> Result<(
    cli::GeneratorOptions,
    Box<dyn generator::Client>,
    Box<dyn generator::Client>,
)> {
    // create generator
    let gen_options = args.to_generator_options()?;
    let generator = generator::factory(
        &gen_options.type_name,
        &gen_options.host,
        gen_options.port,
        &gen_options.model_name,
        gen_options.context_window,
    )?;

    // create embedder
    let emb_options = args.to_embedder_options()?;
    let embedder = generator::factory(
        &emb_options.type_name,
        &emb_options.host,
        emb_options.port,
        &emb_options.model_name,
        emb_options.context_window,
    )?;

    Ok((gen_options, generator, embedder))
}

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: save/restore session

    let args = cli::Args::parse();

    if args.generate_doc {
        // generate action namespaces documentation and exit
        println!("{}", serialization::available_actions());

        return Ok(());
    }

    let tasklet = if let Some(t) = &args.tasklet {
        t
    } else {
        return Err(anyhow!("--tasklet/-T not specified"));
    };

    // create generator and embedder
    let (gen_options, generator, embedder) = setup_models(&args)?;

    // read and create the tasklet
    let mut tasklet: Tasklet = Tasklet::from_path(tasklet, &args.define)?;
    let tasklet_name = tasklet.name.clone();

    println!(
        "{} v{} 🧠 {}{} > {}",
        APP_NAME,
        APP_VERSION,
        gen_options.model_name.bold(),
        if gen_options.port == 0 {
            format!("@{}", gen_options.type_name.dimmed())
        } else {
            format!(
                "@{}:{}",
                gen_options.host.dimmed(),
                gen_options.port.to_string().dimmed()
            )
        },
        tasklet_name.green().bold(),
    );

    tasklet.prepare(&args.prompt)?;

    println!("task: {}\n", tasklet.prompt.as_ref().unwrap().green());

    let task = Box::new(tasklet);

    // create the agent given the generator, task and a set of options
    let mut agent = Agent::new(generator, embedder, task, args.to_agent_options()).await?;

    // keep going until the task is complete or a fatal error is reached
    while !agent.is_done().await {
        // next step
        if let Err(error) = agent.step().await {
            println!("{}", error.to_string().bold().red());
            return Err(error);
        }
    }

    // report final metrics on exit
    if args.stats {
        println!("\n{}", agent.get_metrics().await);
    }

    Ok(())
}
