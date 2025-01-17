use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use crate::agent::{
    self,
    events::{self, create_channel},
    generator::{self, history::ConversationWindow},
    task::{robopages, tasklet::Tasklet},
    Agent,
};

use crate::{cli, APP_NAME, APP_VERSION};

use super::Args;

pub async fn setup_arguments() -> Result<Args> {
    // TODO: save/restore session
    let mut args = cli::Args::parse();

    // set generator url if env variable is set
    if let Ok(env_generator) = std::env::var("NERVE_GENERATOR") {
        args.generator = env_generator;
    } else {
        // set env variable for later use
        std::env::set_var("NERVE_GENERATOR", args.generator.clone());
    }

    // set judge url if env variable is set
    if let Ok(env_judge) = std::env::var("NERVE_JUDGE") {
        args.judge = env_judge;
    } else {
        // set env variable for later use
        std::env::set_var("NERVE_JUDGE", args.judge.clone());
    }

    // if we're running in judge mode, set the generator to the judge model
    if args.judge_mode {
        args.generator = args.judge.clone();
    }

    // set tasklet if env variable is set
    if let Ok(env_tasklet) = std::env::var("NERVE_TASKLET") {
        args.tasklet = Some(env_tasklet);
    }

    // TODO: handle max tokens

    if args.generate_doc {
        // generate action namespaces documentation and exit
        println!("{}", agent::serialization::Strategy::available_actions());
        std::process::exit(0);
    }

    if std::env::var_os("RUST_LOG").is_none() {
        // set `RUST_LOG=debug` to see debug logs
        std::env::set_var(
            "RUST_LOG",
            "info,openai_api_rust=warn,rustls=warn,ureq=warn",
        );
    }

    if args.judge_mode {
        // disable most logging
        std::env::set_var(
            "RUST_LOG",
            "error,openai_api_rust=error,rustls=error,ureq=error",
        );

        // read STDIN and preemptively set $STDIN
        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_goes_into_input_above) => {}
            Err(_no_updates_is_fine) => {}
        }
        agent::task::variables::define_variable("STDIN", input.trim());
    }

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_module_path(false)
        .format_target(false)
        .init();

    Ok(args)
}

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
    )
    .map_err(|e| anyhow!("{}: {}", args.generator, e))?;

    // create embedder
    let emb_options = generator::Options::parse(&args.embedder, args.context_window)?;
    let embedder = generator::factory_embedder(
        &emb_options.type_name,
        &emb_options.host,
        emb_options.port,
        &emb_options.model_name,
        emb_options.context_window,
    )
    .map_err(|e| anyhow!("{}: {}", args.generator, e))?;

    Ok((gen_options, generator, embedder))
}

pub async fn setup_agent_for_task(
    args: &cli::Args,
    workflow_mode: bool,
) -> Result<(Agent, events::Receiver)> {
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
        if !workflow_mode {
            print!("{} v{} 🧠 ", APP_NAME, APP_VERSION);
        }
        println!(
            "{}{} > {} ({})\n",
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
