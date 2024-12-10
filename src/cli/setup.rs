use anyhow::Result;
use colored::Colorize;

use crate::agent::{
    events::{self, create_channel},
    generator::{self, history::ConversationWindow},
    task::{robopages, tasklet::Tasklet},
    Agent,
};

use crate::{cli, APP_NAME, APP_VERSION};

#[allow(clippy::type_complexity)]
fn setup_models(
    args: &cli::Args,
) -> Result<(
    generator::Options,
    Box<dyn generator::Client>,
    Box<dyn mini_rag::Embedder>,
)> {
    // create generator
    let gen_options = generator::Options::parse(&args.generator, args.context_window)?;
    let generator = generator::factory(
        &gen_options.type_name,
        &gen_options.host,
        gen_options.port,
        &gen_options.model_name,
        gen_options.context_window,
    )?;

    // create embedder
    let emb_options = generator::Options::parse(&args.embedder, args.context_window)?;
    let embedder = generator::factory_embedder(
        &emb_options.type_name,
        &emb_options.host,
        emb_options.port,
        &emb_options.model_name,
        emb_options.context_window,
    )?;

    Ok((gen_options, generator, embedder))
}

pub async fn setup_agent(args: &cli::Args) -> Result<(Agent, events::Receiver)> {
    // create generator and embedder
    let (gen_options, generator, embedder) = setup_models(args)?;

    // create the conversation window
    let conversation_window = ConversationWindow::parse(&args.window)?;

    // read and create the tasklet
    let tasklet = if let Some(t) = &args.tasklet {
        t
    } else {
        return Err(anyhow!("--tasklet/-T not specified"));
    };

    let mut tasklet = Tasklet::from_path(tasklet, &args.define)?;
    let tasklet_name = tasklet.name.clone();

    if !args.judge_mode {
        println!(
            "{} v{} 🧠 {}{} > {} ({})\n",
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
            conversation_window.to_string().dimmed(),
        );
    }

    tasklet.prepare(&args.prompt)?;

    if let Some(server_address) = &args.robopages {
        tasklet.set_robopages(
            server_address,
            robopages::Client::new(server_address.to_owned())
                .get_functions()
                .await?,
        );
    }

    let task = Box::new(tasklet);
    let (tx, rx) = create_channel();

    // create the agent
    let agent = Agent::new(
        tx,
        generator,
        embedder,
        task,
        args.serialization.clone(),
        conversation_window,
        args.force_format,
        args.user_only,
        args.max_iterations,
    )
    .await?;

    Ok((agent, rx))
}
