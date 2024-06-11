use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

use anyhow::Result;

lazy_static! {
    pub static ref XML_ATTRIBUTES_PARSER: Regex = Regex::new(r#"(?m)(([^=]+)="([^"]+)")"#).unwrap();
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Invocation {
    pub action: String,
    pub attributes: Option<HashMap<String, String>>,
    pub payload: Option<String>,
    pub xml: String,
}

impl Invocation {
    pub fn new(
        action: String,
        attributes: Option<HashMap<String, String>>,
        payload: Option<String>,
    ) -> Self {
        let mut xml = format!("<{action}");
        if let Some(attrs) = &attributes {
            for (key, value) in attrs {
                xml += &format!(" {key}=\"{value}\"");
            }
        }
        xml += &format!(
            ">{}</{}>",
            if let Some(data) = &payload { data } else { "" },
            action
        );

        Self {
            action,
            attributes,
            payload,
            xml,
        }
    }

    pub fn as_xml(&self) -> &str {
        return self.xml.as_str();
    }
}

pub(crate) fn parse_model_response(model_response: &str) -> Result<Vec<Invocation>> {
    let mut invocations = vec![];

    let model_response_size = model_response.len();
    let mut current = 0;

    // TODO: replace this with a proper xml parser
    while current < model_response_size {
        // read until < or end
        let mut ptr = &model_response[current..];
        if let Some(tag_open_idx) = ptr.find('<') {
            current += tag_open_idx;
            ptr = &ptr[tag_open_idx..];
            // read tag
            if let Some(tag_name_term_idx) = ptr.find(|c: char| c == '>' || c == ' ') {
                current += tag_name_term_idx;
                let tag_name = &ptr[1..tag_name_term_idx];
                // println!("tag_name={}", tag_name);
                if let Some(tag_close_idx) = ptr.find('>') {
                    current += tag_close_idx + tag_name.len();
                    let tag_closing = format!("</{}>", tag_name);
                    let tag_closing_idx = ptr.find(&tag_closing);
                    if let Some(tag_closing_idx) = tag_closing_idx {
                        // parse attributes if any
                        let attributes = if ptr.as_bytes()[tag_name_term_idx] == b' ' {
                            let attr_str = &ptr[tag_name_term_idx + 1..tag_close_idx];
                            let mut attrs = HashMap::new();

                            // parse as a list of key="value"
                            let iter = XML_ATTRIBUTES_PARSER.captures_iter(attr_str);
                            for caps in iter {
                                if caps.len() == 4 {
                                    let key = caps.get(2).unwrap().as_str().trim();
                                    let value = caps.get(3).unwrap().as_str().trim();
                                    attrs.insert(key.to_string(), value.to_string());
                                }
                            }

                            Some(attrs)
                        } else {
                            None
                        };

                        // parse payload if any
                        let after_tag_close = &ptr[tag_close_idx + 1..tag_closing_idx];
                        let payload = if !after_tag_close.is_empty() {
                            if after_tag_close.as_bytes()[0] != b'<' {
                                Some(after_tag_close.trim().to_string())
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        invocations.push(Invocation::new(
                            tag_name.to_string(),
                            attributes,
                            payload,
                        ));

                        continue;
                    }
                }
            }

            // just skip ahead
            current += 1;
        } else {
            // no more tags
            break;
        }
    }

    Ok(invocations)
}
