use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::tasklet::ToolBox;

#[derive(Debug, Serialize, Clone)]
struct FunctionCall {
    pub name: String,
    pub arguments: HashMap<String, String>,
}

#[derive(Debug, Serialize, Clone)]
struct ToolCall {
    pub id: String,
    pub function: FunctionCall,
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub the_type: String,
}

#[derive(Debug, Deserialize, Clone)]
struct CallResult {
    pub content: String,
}

pub struct Client {
    server_address: String,
    server_path: String,
}

impl Client {
    pub fn new(server_address: String) -> Self {
        let server_address = if server_address.contains("://") {
            server_address.split("://").last().unwrap().to_string()
        } else {
            server_address
        };

        // preserve the path part for filtering
        let (server_address, server_path) = if server_address.contains('/') {
            let parts = server_address.split_once("/").unwrap();
            (parts.0.to_string(), format!("/{}", parts.1))
        } else {
            (server_address, "/".to_string())
        };

        Self {
            server_address,
            server_path,
        }
    }

    pub async fn get_functions(&self) -> anyhow::Result<Vec<ToolBox>> {
        let api_url = format!(
            "http://{}{}?flavor=nerve",
            &self.server_address, &self.server_path
        );

        log::info!("fetching robopages from  {} ...", &api_url);

        let response = reqwest::Client::new().get(api_url).send().await?;
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "server returned error status: {}",
                response.status()
            ));
        }

        response
            .json::<Vec<ToolBox>>()
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    pub async fn execute(
        &self,
        function_name: &str,
        arguments: HashMap<String, String>,
    ) -> anyhow::Result<String> {
        let api_url = format!("http://{}/process", &self.server_address);

        log::info!(
            "executing call {}({:?}) via robopages server @ {}",
            function_name,
            &arguments,
            &self.server_address
        );

        // use openai flavor function call flavor
        let tool_call = vec![ToolCall {
            id: "".to_string(),
            the_type: "function".to_string(),
            function: FunctionCall {
                name: function_name.to_string(),
                arguments,
            },
        }];

        let response = reqwest::Client::new()
            .post(&api_url)
            .json(&tool_call)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "server returned error status: {}",
                response.status()
            ));
        }

        let res = response
            .json::<Vec<CallResult>>()
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        if res.is_empty() {
            Err(anyhow::anyhow!("no result returned"))
        } else {
            Ok(res[0].content.clone())
        }
    }
}
