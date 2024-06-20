use std::collections::HashMap;

use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use xml::{reader::XmlEvent, EventReader};

use crate::agent::{
    namespaces::Action,
    state::storage::{Storage, StorageType, CURRENT_TAG, PREVIOUS_TAG},
    Invocation,
};

lazy_static! {
    pub static ref XML_ATTRIBUTES_PARSER: Regex = Regex::new(r#"(?m)(([^=]+)="([^"]+)")"#).unwrap();
}

pub(crate) fn serialize_invocation(inv: &Invocation) -> String {
    let mut xml = format!("<{}", inv.action);
    if let Some(attrs) = &inv.attributes {
        for (key, value) in attrs {
            xml += &format!(" {key}=\"{value}\"");
        }
    }
    xml += &format!(
        ">{}</{}>",
        if let Some(data) = inv.payload.as_ref() {
            data
        } else {
            ""
        },
        inv.action
    );

    xml
}

#[allow(clippy::borrowed_box)]
pub(crate) fn serialize_action(action: &Box<dyn Action>) -> String {
    let mut xml = format!("<{}", action.name());

    if let Some(attrs) = action.attributes() {
        for (name, example_value) in &attrs {
            xml += &format!(" {}=\"{}\"", name, example_value);
        }
    }
    xml += ">";

    if let Some(payload) = action.example_payload() {
        xml += payload; // TODO: escape payload?
    }

    xml += &format!("</{}>", action.name());

    xml
}

pub(crate) fn serialize_storage(storage: &Storage) -> String {
    let inner = storage.get_inner().lock().unwrap();
    if inner.is_empty() {
        return "".to_string();
    }

    match storage.get_type() {
        StorageType::Tagged => {
            let mut xml: String = format!("<{}>\n", storage.get_name());

            for (key, entry) in &*inner {
                xml += &format!("  - {}={}\n", key, &entry.data);
            }

            xml += &format!("</{}>", storage.get_name());

            xml.to_string()
        }
        StorageType::Untagged => {
            let mut xml = format!("<{}>\n", storage.get_name());

            for entry in inner.values() {
                xml += &format!("  - {}\n", &entry.data);
            }

            xml += &format!("</{}>", storage.get_name());

            xml.to_string()
        }
        StorageType::Completion => {
            let mut xml = format!("<{}>\n", storage.get_name());

            for entry in inner.values() {
                xml += &format!(
                    "  - {} : {}\n",
                    &entry.data,
                    if entry.complete {
                        "COMPLETED"
                    } else {
                        "not completed"
                    }
                );
            }

            xml += &format!("</{}>", storage.get_name());

            xml.to_string()
        }
        StorageType::CurrentPrevious => {
            if let Some(current) = inner.get(CURRENT_TAG) {
                let mut str = format!("* Current {}: {}", storage.get_name(), current.data.trim());
                if let Some(prev) = inner.get(PREVIOUS_TAG) {
                    str += &format!("\n* Previous {}: {}", storage.get_name(), prev.data.trim());
                }
                str
            } else {
                "".to_string()
            }
        }
    }
}

#[derive(Default, Debug)]
pub struct Parsed {
    pub processed: usize,
    pub invocations: Vec<Invocation>,
}

fn build_invocation(
    closing_name: String,
    element: &XmlEvent,
    payload: &Option<String>,
) -> Result<Invocation> {
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

    Ok(Invocation::new(action, attributes, payload))
}

fn try_parse_block(ptr: &str) -> Parsed {
    let mut parser = EventReader::from_str(ptr);
    let mut parsed = Parsed::default();
    let src_size = parser.source().len();

    let mut curr_element = None;
    let mut curr_payload = None;

    loop {
        let event = parser.next();
        if let Ok(event) = event {
            // println!("{:?}", &event);
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
                    let ret = build_invocation(
                        name.to_string(),
                        curr_element.as_ref().unwrap(),
                        &curr_payload,
                    );
                    if let Ok(inv) = ret {
                        parsed.invocations.push(inv);
                    } else {
                        eprintln!("WARNING: {:?}", ret.err().unwrap());
                    }
                }
                _ => {
                    eprintln!("WARNING: unexpected xml element: {:?}", event);
                }
            }
        } else {
            break;
        }
    }

    let src_size_now = parser.source().len();

    // amount of successfully processed bytes
    parsed.processed = src_size - src_size_now;

    return parsed;
}

pub(crate) fn try_parse(raw: &str) -> Result<Vec<Invocation>> {
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
            parsed.extend(parsed_block.invocations);

            // update offset
            ptr = &ptr[parsed_block.processed..];
        }
    }

    Ok(parsed)
}

// TODO: add waaaaay more tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_block_infinite_loop() {
        let ptr = "<clear-plan></clear-plan>
<update-goal>test</update-goal>";
        let parsed = try_parse_block(ptr);

        assert_eq!(ptr.len(), parsed.processed);
        assert_eq!(parsed.invocations.len(), 2);

        assert_eq!(&parsed.invocations[0].action, "clear-plan");
        assert_eq!(&parsed.invocations[1].action, "update-goal");
    }
}
