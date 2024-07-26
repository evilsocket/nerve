use ollama_rs::Ollama;

#[tokio::test]
async fn test_embeddings_generation() {
    let ollama = Ollama::default();

    let prompt = "Why is the sky blue?".to_string();

    let res = ollama
        .generate_embeddings("llama2:latest".to_string(), prompt, None)
        .await
        .unwrap();

    dbg!(res);
}
