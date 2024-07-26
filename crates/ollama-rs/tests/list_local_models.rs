#[tokio::test]
async fn test_list_local_models() {
    let ollama = ollama_rs::Ollama::default();

    let models = ollama.list_local_models().await.unwrap();

    dbg!(models);
}
