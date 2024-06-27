use anyhow::Result;
use colored::Colorize;

use crate::{
    agent::{generator, task::tasklet::Tasklet, Agent},
    cli, APP_NAME, APP_VERSION,
};

#[allow(clippy::type_complexity)]
fn setup_models(
    args: &cli::Args,
) -> Result<(
    cli::GeneratorOptions,
    Box<dyn generator::Client>,
    Box<dyn mini_rag::Embedder>,
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
    let embedder = generator::factory_embedder(
        &emb_options.type_name,
        &emb_options.host,
        emb_options.port,
        &emb_options.model_name,
        emb_options.context_window,
    )?;

    Ok((gen_options, generator, embedder))
}

pub(crate) async fn setup_agent(args: &cli::Args) -> Result<Agent> {
    // create generator and embedder
    let (gen_options, generator, embedder) = setup_models(args)?;

    // read and create the tasklet
    let tasklet = if let Some(t) = &args.tasklet {
        t
    } else {
        return Err(anyhow!("--tasklet/-T not specified"));
    };

    let mut tasklet = Tasklet::from_path(tasklet, &args.define)?;
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

    // create the agent given the generator, embedder, task and a set of options
    let agent = Agent::new(generator, embedder, task, args.to_agent_options()).await?;

    Ok(agent)
}
