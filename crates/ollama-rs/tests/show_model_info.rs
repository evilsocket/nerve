#[tokio::test]
async fn test_show_model_info() {
    let ollama = ollama_rs::Ollama::default();

    let model_info = ollama
        .show_model_info("llama2:latest".to_string())
        .await
        .unwrap();

    dbg!(model_info);
}
