# Ollama-rs

### A simple and easy to use library for interacting with the Ollama API.

It was made following the [Ollama API](https://github.com/jmorganca/ollama/blob/main/docs/api.md) documentation.

## Installation

### Add ollama-rs to your Cargo.toml

```toml
[dependencies]
ollama-rs = "0.2.0"
```

### Initialize Ollama

```rust
// By default it will connect to localhost:11434
let ollama = Ollama::default();

// For custom values:
let ollama = Ollama::new("http://localhost".to_string(), 11434);
```

## Usage

Feel free to check the [Chatbot example](https://github.com/pepperoni21/ollama-rs/blob/0.2.0/examples/basic_chatbot.rs) that shows how to use the library to create a simple chatbot in less than 50 lines of code.
You can also check some [other examples](https://github.com/pepperoni21/ollama-rs/tree/0.2.0/examples).

_These examples use poor error handling for simplicity, but you should handle errors properly in your code._

### Completion generation

```rust
let model = "llama2:latest".to_string();
let prompt = "Why is the sky blue?".to_string();

let res = ollama.generate(GenerationRequest::new(model, prompt)).await;

if let Ok(res) = res {
    println!("{}", res.response);
}
```

**OUTPUTS:** _The sky appears blue because of a phenomenon called Rayleigh scattering..._

### Completion generation (streaming)

_Requires the `stream` feature._

```rust
let model = "llama2:latest".to_string();
let prompt = "Why is the sky blue?".to_string();

let mut stream = ollama.generate_stream(GenerationRequest::new(model, prompt)).await.unwrap();

let mut stdout = tokio::io::stdout();
while let Some(res) = stream.next().await {
    let responses = res.unwrap();
    for resp in responses {
        stdout.write(resp.response.as_bytes()).await.unwrap();
        stdout.flush().await.unwrap();
    }
}
```

Same output as above but streamed.

### Completion generation (passing options to the model)

```rust
let model = "llama2:latest".to_string();
let prompt = "Why is the sky blue?".to_string();

let options = GenerationOptions::default()
    .temperature(0.2)
    .repeat_penalty(1.5)
    .top_k(25)
    .top_p(0.25);

let res = ollama.generate(GenerationRequest::new(model, prompt).options(options)).await;

if let Ok(res) = res {
    println!("{}", res.response);
}
```

**OUTPUTS:** _1. Sun emits white sunlight: The sun consists primarily ..._

### Chat mode
Description: _Every message sent and received will be stored in library's history._
_Each time you want to store history, you have to provide an ID for a chat._
_It can be uniq for each user or the same every time, depending on your need_

Example with history:
```rust
let model = "llama2:latest".to_string();
let prompt = "Why is the sky blue?".to_string();
let history_id = "USER_ID_OR_WHATEVER";

let res = ollama
                .send_chat_messages_with_history(
                    ChatMessageRequest::new(
                        model,
                        vec![ChatMessage::user(prompt)], // <- You should provide only one message
                    ),
                    history_id // <- This entry save for us all the history
                ).await;

if let Ok(res) = res {
println!("{}", res.response);
}
```

Getting history for some ID:
```rust
let history_id = "USER_ID_OR_WHATEVER";
let history = ollama.get_message_history(history_id); // <- Option<Vec<ChatMessage>>
// Act
```

Clear history if we no more need it:
```rust
// Clear history for an ID
let history_id = "USER_ID_OR_WHATEVER";
ollama.clear_messages_for_id(history_id);
// Clear history for all chats
ollama.clear_all_messages(); 
```

_Check chat with history examples for [default](https://github.com/pepperoni21/ollama-rs/blob/master/examples/chat_with_history.rs) and [stream](https://github.com/pepperoni21/ollama-rs/blob/master/examples/chat_with_history_stream.rs)_

### List local models

```rust
let res = ollama.list_local_models().await.unwrap();
```

_Returns a vector of `Model` structs._

### Show model information

```rust
let res = ollama.show_model_info("llama2:latest".to_string()).await.unwrap();
```

_Returns a `ModelInfo` struct._

### Create a model

```rust
let res = ollama.create_model(CreateModelRequest::path("model".into(), "/tmp/Modelfile.example".into())).await.unwrap();
```

_Returns a `CreateModelStatus` struct representing the final status of the model creation._

### Create a model (streaming)

_Requires the `stream` feature._

```rust
let mut res = ollama.create_model_stream(CreateModelRequest::path("model".into(), "/tmp/Modelfile.example".into())).await.unwrap();

while let Some(res) = res.next().await {
    let res = res.unwrap();
    // Handle the status
}
```

_Returns a `CreateModelStatusStream` that will stream every status update of the model creation._

### Copy a model

```rust
let _ = ollama.copy_model("mario".into(), "mario_copy".into()).await.unwrap();
```

### Delete a model

```rust
let _ = ollama.delete_model("mario_copy".into()).await.unwrap();
```

### Generate embeddings

```rust
let prompt = "Why is the sky blue?".to_string();
let res = ollama.generate_embeddings("llama2:latest".to_string(), prompt, None).await.unwrap();
```

_Returns a `GenerateEmbeddingsResponse` struct containing the embeddings (a vector of floats)._

### Make a function call

```rust
let tools = vec![Arc::new(Scraper::new())];
let parser = Arc::new(NousFunctionCall::new());
let message = ChatMessage::user("What is the current oil price?".to_string());
let res = ollama.send_function_call(
    FunctionCallRequest::new(
        "adrienbrault/nous-hermes2pro:Q8_0".to_string(),
        tools,
        vec![message],
    ),
    parser,
  ).await.unwrap();
```

_Uses the given tools (such as searching the web) to find an answer, returns a `ChatMessageResponse` with the answer to the question._
