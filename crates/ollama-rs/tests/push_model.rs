use ollama_rs::Ollama;
use tokio_stream::StreamExt;

#[tokio::test]
/// This test needs a local model named `test_model:latest` to work, and requires registering for ollama.ai and adding a public key first.
async fn test_push_model() {
    let ollama = Ollama::default();

    let mut res = ollama
        .push_model_stream("test_model:latest".into(), false)
        .await
        .unwrap();

    while let Some(res) = res.next().await {
        match res {
            Ok(res) => println!("{:?}", res),
            Err(e) => panic!("{:?}", e),
        }
    }
}
