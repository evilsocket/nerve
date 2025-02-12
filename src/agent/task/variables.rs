use std::{collections::HashMap, sync::Mutex};

use anyhow::Result;
use colored::Colorize;
use lazy_static::lazy_static;
use regex::Regex;

use crate::agent::get_user_input;

lazy_static! {
    static ref VAR_CACHE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    static ref VAR_PARSER: Regex = Regex::new(r"(?m)(\$[^\s]+)(\|\|[^\s]+)?").unwrap();
}

pub fn define_variable(name: &str, value: &str) {
    VAR_CACHE
        .lock()
        .unwrap()
        .insert(name.to_owned(), value.to_owned());
}

pub fn get_variable(name: &str) -> Option<String> {
    VAR_CACHE.lock().unwrap().get(name).cloned()
}

pub fn get_variables() -> HashMap<String, String> {
    VAR_CACHE.lock().unwrap().clone()
}

pub fn parse_pre_defined_values(defines: &Vec<String>) -> Result<()> {
    for keyvalue in defines {
        let parts: Vec<&str> = keyvalue.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow!("can't parse {keyvalue}, syntax is: key=value"));
        }

        VAR_CACHE
            .lock()
            .unwrap()
            .insert(parts[0].to_owned(), parts[1].to_owned());
    }

    Ok(())
}

pub async fn interpolate_variables(expr: &str) -> Result<String> {
    let matches = VAR_PARSER.captures_iter(expr);
    let mut interpolated = expr.to_string();

    for m in matches {
        let var_expr = m.get(0).unwrap().as_str();
        let (_, var_value) = parse_variable_expr(var_expr).await?;
        interpolated = interpolated.replace(var_expr, &var_value);
    }

    Ok(interpolated)
}

pub async fn parse_variable_expr(expr: &str) -> Result<(String, String)> {
    if expr.as_bytes()[0] != b'$' {
        return Err(anyhow!("'{}' is not a valid variable expression", expr));
    }

    // variable, the order of lookup is:
    //  0. environment variable
    //  1. cache
    //  2. if an alternative default was provided via || use it
    //  3. ask the user (and cache)
    let var_name = expr.trim_start_matches('$');
    let (var_name, var_default) = if let Some((name, default_value)) = var_name.split_once("||") {
        (name, Some(default_value))
    } else {
        (var_name, None)
    };

    let (var_scheme, var_name) = if let Some((scheme, name)) = var_name.split_once("://") {
        (Some(scheme), name)
    } else {
        (None, var_name)
    };

    let var_value = match var_scheme {
        // no scheme, get from env, cache, default or from user
        None => {
            let mut var_cache = VAR_CACHE.lock().unwrap();
            if let Ok(value) = std::env::var(var_name) {
                // get from env
                value
            } else if let Some(cached) = var_cache.get(var_name) {
                // get from cached
                cached.to_string()
            } else if let Some(var_default) = var_default {
                // get from default
                var_default.to_string()
            } else {
                // get from user
                let var_value = get_user_input(&format!("\nplease set ${}: ", var_name.yellow()));
                var_cache.insert(var_name.to_owned(), var_value.clone());
                var_value
            }
        }
        // read from file or default if set
        Some("file") => {
            let path = std::path::PathBuf::from(var_name);
            let ret = std::fs::read_to_string(&path);
            if let Ok(content) = ret {
                content
            } else if let Some(default) = var_default {
                default.to_string()
            } else {
                return Err(anyhow!("file not found: {}", var_name));
            }
        }
        // read from url or default if set
        Some("http") | Some("https") => {
            if let Ok(response) =
                reqwest::get(format!("{}://{}", var_scheme.unwrap(), var_name)).await
            {
                response.text().await?
            } else if let Some(default) = var_default {
                default.to_string()
            } else {
                return Err(anyhow!("failed to fetch from url: {}", var_name));
            }
        }
        _ => {
            return Err(anyhow!("unsupported scheme: {}", var_scheme.unwrap()));
        }
    };

    Ok((var_name.to_string(), var_value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_env_variable_interpolation() {
        std::env::set_var("TEST_VAR", "test_value");
        assert_eq!(
            interpolate_variables("Value is $TEST_VAR").await.unwrap(),
            "Value is test_value"
        );
        std::env::remove_var("TEST_VAR");
    }

    #[tokio::test]
    async fn test_default_value_interpolation() {
        assert_eq!(
            interpolate_variables("Value is $NONEXISTENT||default")
                .await
                .unwrap(),
            "Value is default"
        );
    }

    #[tokio::test]
    async fn test_multiple_variables() {
        std::env::set_var("VAR1", "first");
        std::env::set_var("VAR2", "second");
        assert_eq!(
            interpolate_variables("$VAR1 and $VAR2").await.unwrap(),
            "first and second"
        );
        std::env::remove_var("VAR1");
        std::env::remove_var("VAR2");
    }

    #[tokio::test]
    async fn test_cached_variable() {
        {
            let mut cache = VAR_CACHE.lock().unwrap();
            cache.insert("CACHED_VAR".to_string(), "cached_value".to_string());
        }
        assert_eq!(
            interpolate_variables("$CACHED_VAR").await.unwrap(),
            "cached_value"
        );
        {
            let mut cache = VAR_CACHE.lock().unwrap();
            cache.clear();
        }
    }

    #[tokio::test]
    async fn test_no_variables() {
        assert_eq!(
            interpolate_variables("Plain text without variables")
                .await
                .unwrap(),
            "Plain text without variables"
        );
    }

    #[tokio::test]
    async fn test_interpolation_with_non_existing_schema() {
        let ret = interpolate_variables("Value is $foo://bar");
        assert!(ret.await.is_err());
    }

    #[tokio::test]
    async fn test_file_interpolation_with_non_existing_file() {
        let ret = interpolate_variables("Value is $file:///idonotexist.txt");
        assert!(ret.await.is_err());
    }

    #[tokio::test]
    async fn test_file_interpolation_with_non_existing_file_with_default() {
        let ret = interpolate_variables("Value is $file:///idonotexist.txt||ok");
        assert_eq!(ret.await.unwrap(), "Value is ok");
    }

    #[tokio::test]
    async fn test_http_interpolation() {
        let mock_server = wiremock::MockServer::start().await;

        wiremock::Mock::given(wiremock::matchers::method("GET"))
            .and(wiremock::matchers::path("/test"))
            .respond_with(wiremock::ResponseTemplate::new(200).set_body_string("hello"))
            .mount(&mock_server)
            .await;

        assert_eq!(
            interpolate_variables(&format!("Value is $http://{}/test", mock_server.address()))
                .await
                .unwrap(),
            "Value is hello"
        );
    }

    #[tokio::test]
    async fn test_http_interpolation_with_non_existing_url() {
        let ret = interpolate_variables("Value is $http://localhost:1234/notfound");
        assert!(ret.await.is_err());
    }

    #[tokio::test]
    async fn test_http_interpolation_with_default() {
        let ret = interpolate_variables("Value is $http://localhost:1234/notfound||default");
        assert_eq!(ret.await.unwrap(), "Value is default");
    }
}
