#[tokio::test]
/// This test needs a model named "mario" to work
async fn test_copy_model() {
    let ollama = ollama_rs::Ollama::default();

    ollama
        .copy_model("mario".into(), "mario_copy".into())
        .await
        .unwrap();
}
