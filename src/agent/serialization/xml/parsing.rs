use std::collections::HashMap;

use anyhow::Result;
use itertools::Itertools;
use xml::{reader::XmlEvent, EventReader};

use crate::agent::ToolCall;

#[derive(Default, Debug)]
pub struct Parsed {
    pub processed: usize,
    pub tool_calls: Vec<ToolCall>,
}

fn build_tool_call(
    closing_name: String,
    element: &XmlEvent,
    payload: &Option<String>,
) -> Result<ToolCall> {
    let (name, attrs) = match element {
        xml::reader::XmlEvent::StartElement {
            name,
            attributes,
            namespace: _,
        } => (name.to_string(), attributes),
        _ => {
            return Err(anyhow!("unexpected element {:?}", element));
        }
    };

    if name != closing_name {
        return Err(anyhow!(
            "unexpected closing {} while parsing {}",
            closing_name,
            name
        ));
    }

    let action = name.to_string();
    let attributes = if !attrs.is_empty() {
        let mut map = HashMap::new();
        for attr in attrs {
            map.insert(attr.name.to_string(), attr.value.to_string());
        }

        Some(map)
    } else {
        None
    };
    let payload = payload.as_ref().map(|data| data.to_owned());

    Ok(ToolCall::new(action, attributes, payload))
}

fn preprocess_block(ptr: &str) -> String {
    if ptr.len() > 2 {
        assert_eq!(ptr.as_bytes()[0], b'<');
        // not a closing tag
        if ptr.as_bytes()[1] != b'/' {
            // determine tag name
            let tag_name = &ptr[1..ptr.find([' ', '>']).unwrap()];
            let payload_start_idx = ptr.find('>').unwrap();
            // if not a short <tag/>
            if !tag_name.ends_with('/') {
                // estimate tag closing index and get payload
                let tag_closing = format!("</{}>", &tag_name);
                if let Some(tag_closing_idx) = ptr.find(&tag_closing) {
                    let from = payload_start_idx + 1;
                    let to = tag_closing_idx;
                    // valid xml?
                    if to > from {
                        let payload = &ptr[payload_start_idx + 1..tag_closing_idx];
                        if !payload.is_empty() {
                            // if escaped payload is different, replace it
                            let escaped = xml::escape::escape_str_pcdata(payload);
                            if escaped != payload {
                                return ptr.replace(payload, &escaped);
                            }
                        }
                    }
                }
            }
        }
    }

    ptr.to_string()
}

fn try_parse_block(ptr: &str) -> Parsed {
    // we need some preprocessing to handle unquoted characters
    let prev = ptr.len();
    let ptr = preprocess_block(ptr);
    let delta = if ptr.len() != prev {
        // some escaping happened, account for this is number of processed chars
        ptr.len() - prev
    } else {
        0
    };

    let mut parser = EventReader::from_str(&ptr);
    let mut parsed = Parsed::default();
    let src_size = parser.source().len();

    let mut curr_element = None;
    let mut curr_payload = None;

    loop {
        let event = parser.next();
        if let Ok(event) = event {
            log::debug!("{:?}", &event);
            match event {
                xml::reader::XmlEvent::StartDocument {
                    version: _,
                    encoding: _,
                    standalone: _,
                } => {}
                xml::reader::XmlEvent::EndDocument {} => {
                    break;
                }
                xml::reader::XmlEvent::StartElement {
                    name: _,
                    attributes: _,
                    namespace: _,
                } => {
                    curr_element = Some(event);
                }
                xml::reader::XmlEvent::Characters(data) => {
                    curr_payload = Some(data);
                }
                xml::reader::XmlEvent::EndElement { name } => {
                    let ret = build_tool_call(
                        name.to_string(),
                        curr_element.as_ref().unwrap(),
                        &curr_payload,
                    );
                    if let Ok(call) = ret {
                        parsed.tool_calls.push(call);
                    } else {
                        log::error!("{:?}", ret.err().unwrap());
                    }
                    break;
                }
                _ => {
                    log::error!("unexpected xml element: {:?}", event);
                }
            }
        } else {
            break;
        }
    }

    let src_size_now = parser.source().len();

    // amount of successfully processed bytes
    parsed.processed = src_size - src_size_now - delta;

    parsed
}

pub(crate) fn try_parse(raw: &str) -> Result<Vec<ToolCall>> {
    let mut ptr = raw;
    let mut parsed = vec![];

    loop {
        // search for a potential xml opening
        let open_idx = ptr.find('<');
        if open_idx.is_none() {
            // no more xml
            break;
        }

        let xml_start = open_idx.unwrap();
        ptr = &ptr[xml_start..];

        let parsed_block = try_parse_block(ptr);
        if parsed_block.processed == 0 {
            break;
        } else {
            parsed.extend(parsed_block.tool_calls);

            // update offset
            ptr = &ptr[parsed_block.processed..];
        }
    }

    // avoid running the same command twince in a row
    Ok(parsed.into_iter().unique().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let ptr = "<clear-plan></clear-plan>";
        let parsed = try_parse_block(ptr);

        assert_eq!(ptr.len(), parsed.processed);
        assert_eq!(parsed.tool_calls.len(), 1);

        assert_eq!(&parsed.tool_calls[0].tool_name, "clear-plan");
        assert_eq!(&parsed.tool_calls[0].argument, &None);
        assert_eq!(&parsed.tool_calls[0].named_arguments, &None);
    }

    #[test]
    fn test_parse_short() {
        let ptr = "<yo/>";
        let parsed = try_parse_block(ptr);

        assert_eq!(ptr.len(), parsed.processed);
        assert_eq!(parsed.tool_calls.len(), 1);

        assert_eq!(&parsed.tool_calls[0].tool_name, "yo");
        assert_eq!(&parsed.tool_calls[0].argument, &None);
        assert_eq!(&parsed.tool_calls[0].named_arguments, &None);
    }

    #[test]
    fn test_parse_payload() {
        let ptr = "<do>this!</do>";
        let parsed = try_parse_block(ptr);

        assert_eq!(ptr.len(), parsed.processed);
        assert_eq!(parsed.tool_calls.len(), 1);

        assert_eq!(&parsed.tool_calls[0].tool_name, "do");
        assert_eq!(parsed.tool_calls[0].argument, Some("this!".to_string()));
        assert_eq!(&parsed.tool_calls[0].named_arguments, &None);
    }

    #[test]
    fn test_parse_attributes() {
        let ptr = "<do foo=\"bar\">this!</do>";
        let parsed = try_parse_block(ptr);

        let attrs = {
            let mut m = HashMap::new();
            m.insert("foo".to_string(), "bar".to_string());
            m
        };

        assert_eq!(ptr.len(), parsed.processed);
        assert_eq!(parsed.tool_calls.len(), 1);

        assert_eq!(&parsed.tool_calls[0].tool_name, "do");
        assert_eq!(parsed.tool_calls[0].argument, Some("this!".to_string()));
        assert_eq!(parsed.tool_calls[0].named_arguments, Some(attrs));
    }

    #[test]
    fn test_parse_mixed_stuff() {
        let ptr = "irhg3984h92fh4f2 <do foo=\"bar\">this!</do> no! whaaaaat, nope ok <clear-plan></clear-plan> and then <do/> ... or not!";
        let tool_calls = try_parse(ptr).unwrap();

        let attrs = {
            let mut m = HashMap::new();
            m.insert("foo".to_string(), "bar".to_string());
            m
        };

        assert_eq!(tool_calls.len(), 3);

        assert_eq!(&tool_calls[0].tool_name, "do");
        assert_eq!(tool_calls[0].argument, Some("this!".to_string()));
        assert_eq!(tool_calls[0].named_arguments, Some(attrs));

        assert_eq!(&tool_calls[1].tool_name, "clear-plan");
        assert_eq!(&tool_calls[1].argument, &None);
        assert_eq!(&tool_calls[1].named_arguments, &None);

        assert_eq!(&tool_calls[2].tool_name, "do");
        assert_eq!(&tool_calls[2].argument, &None);
        assert_eq!(&tool_calls[2].named_arguments, &None);
    }

    #[test]
    fn test_parse_multiple_with_newline() {
        let ptr = "<clear-plan></clear-plan>
<update-goal>test</update-goal>";
        let tool_calls = try_parse(ptr).unwrap();

        assert_eq!(tool_calls.len(), 2);

        assert_eq!(&tool_calls[0].tool_name, "clear-plan");
        assert_eq!(&tool_calls[1].tool_name, "update-goal");
    }

    #[test]
    fn test_parse_unquoted() {
        let ptr = "<command>ls -la && pwd</command>  <other>yes < no</other>";
        let tool_calls = try_parse(ptr).unwrap();
        assert_eq!(tool_calls.len(), 2);

        assert_eq!(&tool_calls[0].tool_name, "command");
        assert_eq!(tool_calls[0].argument, Some("ls -la && pwd".to_string()));
        assert_eq!(&tool_calls[1].tool_name, "other");
        assert_eq!(tool_calls[1].argument, Some("yes < no".to_string()));
    }

    #[test]
    fn test_preprocess_broken_block() {
        let block = "<search site:bing.com Darmepinter</search>";
        let preprocessed = preprocess_block(block);

        assert_eq!(block, &preprocessed);
    }
}
