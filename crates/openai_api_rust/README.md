# OpenAI API for Rust

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/openai-rs/openai-api/rust.yml?style=flat-square)](https://github.com/openai-rs/openai-api/actions)
[![Crates.io](https://img.shields.io/crates/v/openai_api_rust?style=flat-square)](https://crates.io/crates/openai_api_rust/versions)
[![Crates.io](https://img.shields.io/crates/d/openai_api_rust?style=flat-square)](https://crates.io/crates/openai_api_rust)
[![GitHub](https://img.shields.io/github/license/openai-rs/openai-api?style=flat-square)](https://github.com/openai-rs/openai-api/blob/main/LICENSE)

A community-maintained library provides a simple and convenient way to interact with the OpenAI API.
No complex async and redundant dependencies.

## API

check [official API reference](https://platform.openai.com/docs/api-reference)
|API|Support|
|---|---|
|Models|✔️|
|Completions|✔️|
|Chat|✔️|
|Images|✔️|
|Embeddings|✔️|
|Audio|✔️|
|Files|❌|
|Fine-tunes|❌|
|Moderations|❌|
|Engines|❌|
___

## Usage

Add the following to your Cargo.toml file:

```toml
openai_api_rust = "0.1.9"
```

Export your API key into the environment variables

```bash
export OPENAI_API_KEY=<your_api_key>
```

Then use the crate in your Rust code:

```rust
use openai_api_rust::*;
use openai_api_rust::chat::*;
use openai_api_rust::completions::*;

fn main() {
    // Load API key from environment OPENAI_API_KEY.
    // You can also hadcode through `Auth::new(<your_api_key>)`, but it is not recommended.
    let auth = Auth::from_env().unwrap();
    let openai = OpenAI::new(auth, "https://api.openai.com/v1/");
    let body = ChatBody {
        model: "gpt-3.5-turbo".to_string(),
        max_tokens: Some(7),
        temperature: Some(0_f32),
        top_p: Some(0_f32),
        n: Some(2),
        stream: Some(false),
        stop: None,
        presence_penalty: None,
        frequency_penalty: None,
        logit_bias: None,
        user: None,
        messages: vec![Message { role: Role::User, content: "Hello!".to_string() }],
    };
    let rs = openai.chat_completion_create(&body);
    let choice = rs.unwrap().choices;
    let message = &choice[0].message.as_ref().unwrap();
    assert!(message.content.contains("Hello"));
}
```

### Use proxy

Load proxy from env

```rust
let openai = OpenAI::new(auth, "https://api.openai.com/v1/")
        .use_env_proxy();
```

Set the proxy manually

```rust
let openai = OpenAI::new(auth, "https://api.openai.com/v1/")
        .set_proxy("http://127.0.0.1:1080");
```

## License

This library is distributed under the terms of the MIT license. See [LICENSE](LICENSE) for details.
