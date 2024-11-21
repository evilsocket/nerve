use std::{collections::HashMap, sync::Mutex};

use anyhow::Result;
use colored::Colorize;
use lazy_static::lazy_static;
use regex::Regex;

use crate::agent::get_user_input;

lazy_static! {
    static ref VAR_CACHE: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    static ref VAR_PARSER: Regex =
        Regex::new(r"(?m)(\$[A-Za-z][A-Za-z0-9_]+)(\|\|[^\s]+)?").unwrap();
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

pub fn interpolate_variables(expr: &str) -> Result<String> {
    let matches = VAR_PARSER.captures_iter(expr);
    let mut interpolated = expr.to_string();

    for m in matches {
        let var_expr = m.get(0).unwrap().as_str();
        let (_, var_value) = parse_variable_expr(var_expr)?;
        interpolated = interpolated.replace(var_expr, &var_value);
    }

    Ok(interpolated)
}

pub fn parse_variable_expr(expr: &str) -> Result<(String, String)> {
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

    let mut var_cache = VAR_CACHE.lock().unwrap();
    let var_value = if let Ok(value) = std::env::var(var_name) {
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
    };

    Ok((var_name.to_string(), var_value))
}
