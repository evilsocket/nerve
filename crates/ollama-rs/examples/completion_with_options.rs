use ollama_rs::{
    generation::{completion::request::GenerationRequest, options::GenerationOptions},
    Ollama,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ollama = Ollama::default();
    let model = "llama2:latest".to_string();
    let prompt = "Why is the sky blue?".to_string();

    let options = GenerationOptions::default()
        .temperature(0.2)
        .repeat_penalty(1.5)
        .top_k(25)
        .top_p(0.25);

    let res = ollama
        .generate(GenerationRequest::new(model, prompt).options(options))
        .await;

    if let Ok(res) = res {
        println!("{}", res.response);
    }
    Ok(())
}
