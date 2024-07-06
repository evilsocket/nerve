use std::{
    collections::HashMap,
    str::FromStr,
    time::{Duration, Instant},
};

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use url::Url;

use crate::agent::state::SharedState;

use super::{Action, Namespace, StorageDescriptor};

#[derive(Debug, Default, Clone)]
struct ClearHeaders {}

#[async_trait]
impl Action for ClearHeaders {
    fn name(&self) -> &str {
        "http-clear-headers"
    }

    fn description(&self) -> &str {
        include_str!("clear-headers.prompt")
    }

    async fn run(
        &self,
        state: SharedState,
        _: Option<HashMap<String, String>>,
        _: Option<String>,
    ) -> Result<Option<String>> {
        state.lock().await.get_storage_mut("http-headers")?.clear();
        Ok(Some("http headers cleared".to_string()))
    }
}

#[derive(Debug, Default, Clone)]
struct SetHeader {}

#[async_trait]
impl Action for SetHeader {
    fn name(&self) -> &str {
        "http-set-header"
    }

    fn description(&self) -> &str {
        include_str!("set-header.prompt")
    }

    fn example_attributes(&self) -> Option<HashMap<String, String>> {
        let mut attributes = HashMap::new();

        attributes.insert("name".to_string(), "X-Header".to_string());

        Some(attributes)
    }

    fn example_payload(&self) -> Option<&str> {
        Some("some-value-for-the-header")
    }

    async fn run(
        &self,
        state: SharedState,
        attrs: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        let attrs = attrs.unwrap();
        let key = attrs.get("name").unwrap();
        let data = payload.unwrap();

        state
            .lock()
            .await
            .get_storage_mut("http-headers")?
            .add_tagged(key, &data);

        Ok(Some("header set".to_string()))
    }
}

#[derive(Debug, Default, Clone)]
struct Request {}

impl Request {
    async fn create_target_url_from(state: &SharedState, payload: Option<String>) -> Result<Url> {
        let req_page = payload.unwrap();
        let lock = state.lock().await;
        let mut http_target = if let Some(val) = lock.get_variable("HTTP_TARGET") {
            val.to_owned()
        } else {
            return Err(anyhow!("HTTP_TARGET not defined"));
        };

        // add schema if not present
        if !http_target.contains("://") {
            http_target = format!("http://{http_target}");
        }

        Url::parse(&http_target)
            .map_err(|e| anyhow!("can't parse {http_target}: {e}"))?
            .join(&req_page)
            .map_err(|e| anyhow!("can't join {req_page} to {http_target}: {e}"))
    }

    async fn handle_success_response(res: reqwest::Response) -> Result<(String, String)> {
        let reason = res.status().canonical_reason().unwrap();
        let mut resp = format!("{} {}\n", res.status().as_u16(), &reason);

        for (key, val) in res.headers() {
            resp += &format!("{}: {}\n", key, val.to_str()?);
        }

        resp += "\n\n";

        // handle the response according to its content-type
        let content_type = res.headers().get("content-type");
        if let Some(content_type) = content_type {
            let content_type = content_type.to_str()?;
            if content_type == "application/octet-stream" {
                // download the first few bytes to determine if it's binary or text
                let partial_content = res.bytes().await?;
                let is_binary = partial_content
                    .iter()
                    .any(|&byte| byte == 0 || (byte < 32 && byte != 9 && byte != 10 && byte != 13));
                if is_binary {
                    log::warn!(
                        "ignoring binary data with http content type: application/octet-stream"
                    );
                    resp += "<BINARY DATA>";
                } else {
                    resp += std::str::from_utf8(&partial_content).unwrap_or("");
                }
            } else if content_type.starts_with("application/") || content_type.starts_with("text/")
            {
                resp += &res.text().await?;
            } else {
                log::warn!("ignoring non-textual http content type: {}", content_type);
                resp += "<BINARY DATA>";
            }
        } else {
            log::warn!("no content type specified in the http response");
            resp += "<BINARY DATA>";
        }

        Ok((reason.to_string(), resp))
    }
}

#[async_trait]
impl Action for Request {
    fn name(&self) -> &str {
        "http-request"
    }

    fn description(&self) -> &str {
        include_str!("request.prompt")
    }

    fn timeout(&self) -> Option<Duration> {
        Some(Duration::from_secs(30))
    }

    fn example_payload(&self) -> Option<&str> {
        Some("/index.php?id=1")
    }

    fn example_attributes(&self) -> Option<HashMap<String, String>> {
        let mut attributes = HashMap::new();

        attributes.insert("method".to_string(), "GET".to_string());

        Some(attributes)
    }

    fn required_variables(&self) -> Option<Vec<String>> {
        Some(vec!["HTTP_TARGET".to_string()])
    }

    async fn run(
        &self,
        state: SharedState,
        attrs: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Result<Option<String>> {
        // create a parsed Url from the attributes, payload and HTTP_TARGET variable
        let attrs = attrs.unwrap();
        let method = reqwest::Method::from_str(attrs.get("method").unwrap())?;
        let target_url = Self::create_target_url_from(&state, payload.clone()).await?;
        let query_str = target_url.query().unwrap_or("").to_string();

        // TODO: handle cookie/session persistency

        let mut request = reqwest::Client::new().request(method.clone(), target_url.clone());

        // add defined headers
        for (key, value) in state.lock().await.get_storage("http-headers")?.iter() {
            request = request.header(key, &value.data);
        }

        // if there're parameters and we're not in GET, set them as the body
        if !query_str.is_empty() && !matches!(method, reqwest::Method::GET) {
            request = request.header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            );
            request = request.body(query_str);
        }

        log::info!(
            "{}.{} {} ...",
            "http".bold(),
            method.to_string().yellow(),
            target_url.to_string(),
        );

        // perform the request
        let start = Instant::now();
        let res = request.send().await?;
        let elaps = start.elapsed();

        return if res.status().is_success() {
            let (reason, resp) = Self::handle_success_response(res).await?;
            log::info!(
                "   {} {} -> {} bytes",
                reason.green(),
                format!("({:?})", elaps).dimmed(),
                resp.len()
            );

            Ok(Some(resp))
        } else {
            let reason = res.status().canonical_reason().unwrap();
            let resp = format!("{} {}", res.status().as_u16(), &reason);

            log::error!("   {} {}", reason.red(), format!("({:?})", elaps).dimmed(),);

            Err(anyhow!(resp))
        };
    }
}

pub(crate) fn get_namespace() -> Namespace {
    let mut predefined_headers = HashMap::new();

    predefined_headers.insert("User-Agent".to_string(), "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36".to_string());
    predefined_headers.insert("Accept-Encoding".to_string(), "deflate".to_string());

    Namespace::new_non_default(
        "Web".to_string(),
        include_str!("ns.prompt").to_string(),
        vec![
            Box::<SetHeader>::default(),
            Box::<ClearHeaders>::default(),
            Box::<Request>::default(),
        ],
        Some(vec![
            StorageDescriptor::tagged("http-headers").predefine(predefined_headers)
        ]),
    )
}
