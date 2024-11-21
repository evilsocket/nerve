use std::{
    collections::HashMap,
    fs::OpenOptions,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::Result;
use async_trait::async_trait;
use colored::Colorize;
use lazy_static::lazy_static;
use reqwest_cookie_store::CookieStoreMutex;
use url::Url;

use crate::agent::state::SharedState;

use super::{Action, Namespace, StorageDescriptor};

const DEFAULT_HTTP_SCHEMA: &str = "https";

lazy_static! {
    static ref COOKIE_STORE: Arc<CookieStoreMutex> = {
        let cookies_file = crate::agent::data_path("http")
            .unwrap()
            .join("cookies.json");

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&cookies_file)
            .map(std::io::BufReader::new)
            .unwrap_or_else(|_| panic!("can't open {}", cookies_file.display()));

        let cookie_store = reqwest_cookie_store::CookieStore::load_json(file)
            .unwrap_or_else(|_| panic!("can't load {}", cookies_file.display()));

        Arc::new(reqwest_cookie_store::CookieStoreMutex::new(cookie_store))
    };
}

#[derive(Debug, Default, Clone)]
struct ClearHeaders {}

#[async_trait]
impl Action for ClearHeaders {
    fn name(&self) -> &str {
        "http_clear_headers"
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
        "http_set_header"
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
            http_target = format!("{DEFAULT_HTTP_SCHEMA}://{http_target}");
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

    fn create_request(method: &str, target_url: Url) -> Result<reqwest::RequestBuilder> {
        let method = reqwest::Method::from_str(method)?;

        let mut request = reqwest::Client::builder()
            .cookie_provider(COOKIE_STORE.clone())
            .build()?
            .request(method.clone(), target_url.clone());

        // get query string if any
        let query_str = target_url.query().unwrap_or("").to_string();
        // if there're parameters and we're not in GET, set them as the body
        if !query_str.is_empty() && !matches!(method, reqwest::Method::GET) {
            request = request.header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded",
            );
            request = request.body(query_str);
        }

        Ok(request)
    }
}

#[async_trait]
impl Action for Request {
    fn name(&self) -> &str {
        "http_request"
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
        let method = attrs.get("method").unwrap();
        let target_url = Self::create_target_url_from(&state, payload.clone()).await?;
        let target_url_str = target_url.to_string();
        let mut request = Self::create_request(method, target_url)?;

        // add defined headers
        for (key, value) in state.lock().await.get_storage("http-headers")?.iter() {
            request = request.header(key, &value.data);
        }

        log::debug!(
            "{}.{} {} ...",
            "http".bold(),
            method.to_string().yellow(),
            target_url_str,
        );

        // perform the request
        let start = Instant::now();
        let res = request.send().await?;
        let elaps = start.elapsed();

        return if res.status().is_success() {
            let (reason, resp) = Self::handle_success_response(res).await?;
            log::debug!(
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

pub fn get_namespace() -> Namespace {
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::agent::state::State;

    use super::*;

    #[derive(Debug)]
    struct TestTask {}

    impl crate::agent::task::Task for TestTask {
        fn to_system_prompt(&self) -> Result<String> {
            Ok("test".to_string())
        }

        fn to_prompt(&self) -> Result<String> {
            Ok("test".to_string())
        }

        fn get_functions(&self) -> Vec<Namespace> {
            vec![]
        }
    }

    struct TestEmbedder {}

    #[async_trait]
    impl mini_rag::Embedder for TestEmbedder {
        async fn embed(&self, _text: &str) -> Result<mini_rag::Embeddings> {
            todo!()
        }
    }

    #[allow(unused_variables)]
    async fn create_test_state(vars: Vec<(String, String)>) -> Result<SharedState> {
        let (tx, _rx) = crate::agent::events::create_channel();

        let task = Box::new(TestTask {});
        let embedder = Box::new(TestEmbedder {});

        let mut state = State::new(tx, task, embedder, 10, false).await?;

        for (name, value) in vars {
            state.set_variable(name, value);
        }

        Ok(Arc::new(tokio::sync::Mutex::new(state)))
    }

    #[tokio::test]
    async fn test_parse_no_target() {
        let state = create_test_state(vec![]).await.unwrap();
        let payload = Some("/".to_string());
        let target_url = Request::create_target_url_from(&state, payload.clone()).await;

        assert!(target_url.is_err());
    }

    #[tokio::test]
    async fn test_parse_simple_get_without_schema() {
        let state = create_test_state(vec![(
            "HTTP_TARGET".to_string(),
            "www.example.com".to_string(),
        )])
        .await
        .unwrap();

        let payload = Some("/".to_string());
        let target_url = Request::create_target_url_from(&state, payload.clone())
            .await
            .unwrap();

        assert_eq!(
            target_url.to_string(),
            format!("{DEFAULT_HTTP_SCHEMA}://www.example.com/")
        );
    }

    #[tokio::test]
    async fn test_parse_simple_get_with_schema() {
        let state = create_test_state(vec![(
            "HTTP_TARGET".to_string(),
            "ftp://www.example.com".to_string(),
        )])
        .await
        .unwrap();

        let payload = Some("/".to_string());
        let target_url = Request::create_target_url_from(&state, payload.clone())
            .await
            .unwrap();

        assert_eq!(target_url.to_string(), format!("ftp://www.example.com/"));
    }

    #[tokio::test]
    async fn test_parse_simple_get_with_schema_and_port() {
        let state = create_test_state(vec![(
            "HTTP_TARGET".to_string(),
            "ftp://www.example.com:1012".to_string(),
        )])
        .await
        .unwrap();

        let payload = Some("/".to_string());
        let target_url = Request::create_target_url_from(&state, payload.clone())
            .await
            .unwrap();

        assert_eq!(
            target_url.to_string(),
            format!("ftp://www.example.com:1012/")
        );
    }

    #[tokio::test]
    async fn test_parse_query_string() {
        let state = create_test_state(vec![(
            "HTTP_TARGET".to_string(),
            "www.example.com".to_string(),
        )])
        .await
        .unwrap();

        let payload = Some("/index.php?id=1&name=foo".to_string());
        let target_url = Request::create_target_url_from(&state, payload.clone())
            .await
            .unwrap();

        assert_eq!(
            target_url.to_string(),
            format!("{DEFAULT_HTTP_SCHEMA}://www.example.com/index.php?id=1&name=foo")
        );
    }

    #[tokio::test]
    async fn test_parse_query_string_is_escaped() {
        let state = create_test_state(vec![(
            "HTTP_TARGET".to_string(),
            "www.example.com".to_string(),
        )])
        .await
        .unwrap();

        let payload = Some("/index.php?id=1&name=foo' or ''='".to_string());
        let target_url = Request::create_target_url_from(&state, payload.clone())
            .await
            .unwrap();

        assert_eq!(
            target_url.to_string(),
            format!("{DEFAULT_HTTP_SCHEMA}://www.example.com/index.php?id=1&name=foo%27%20or%20%27%27=%27")
        );
    }
    #[tokio::test]
    async fn test_parse_body_post() {
        let state = create_test_state(vec![(
            "HTTP_TARGET".to_string(),
            "www.example.com".to_string(),
        )])
        .await
        .unwrap();

        let method = "POST";
        let payload = Some("/login.php?user=admin&pass=' OR ''='".to_string());
        let target_url = Request::create_target_url_from(&state, payload.clone())
            .await
            .unwrap();
        let expected_body_string = "user=admin&pass=%27%20OR%20%27%27=%27".to_string();
        let expected_target_url_string = format!(
            "{DEFAULT_HTTP_SCHEMA}://www.example.com/login.php?{}",
            expected_body_string
        );

        assert_eq!(target_url.to_string(), expected_target_url_string);

        let request = Request::create_request(method, target_url)
            .unwrap()
            .build()
            .unwrap();

        assert_eq!(request.method().to_string(), method.to_string());
        assert_eq!(request.url().to_string(), expected_target_url_string);
        assert!(request.body().is_some());
        assert_eq!(
            request.body().unwrap().as_bytes(),
            Some(expected_body_string.as_bytes())
        );
    }
}
