use ollama_rs::{
    generation::{completion::request::GenerationRequest, options::GenerationOptions},
    Ollama,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ollama = Ollama::default();
    let model = "llama2:latest".to_string();
    let prompt = "Why is the sky blue?".to_string();

    // Fetch the configuration from a file or from user request
    // let options_str = fs::read_to_string("options.json").expect("The option file should be available")    ;
    let options_str = r#"{
      "temperature": 0.2,
      "repeat_penalty": 1.5,
      "top_k": 25,
      "top_p": 0.25
    }"#;
    let options: GenerationOptions =
        serde_json::from_str(options_str).expect("JSON was not well-formatted");
    let res = ollama
        .generate(GenerationRequest::new(model, prompt).options(options))
        .await;

    if let Ok(res) = res {
        println!("{}", res.response);
    }
    Ok(())
}
