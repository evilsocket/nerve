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

    // TODO: investigate CUDA crashes for low context window sizes.
    // An error occurred with ollama-rs: {"error":"an unknown error was encountered while running the model CUDA error: an illegal memory access was encountered\n  current device: 0, in function ggml_backend_cuda_synchronize at /go/src/github.com/ollama/ollama/llm/llama.cpp/ggml-cuda.cu:2463\n  cudaStreamSynchronize(cuda_ctx-\u003estream())\nGGML_ASSERT: /go/src/github.com/ollama/ollama/llm/llama.cpp/ggml-cuda.cu:100: !\"CUDA error\""}
    // create generator
    let gen_options = args.to_generator_options()?;
    let generator = generator::factory(
        &gen_options.type_name,
        &gen_options.host,
        gen_options.port,
        &gen_options.model_name,
        gen_options.context_window,
    )?;

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
    let mut agent = Agent::new(generator, task, args.to_agent_options())?;

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
