use std::collections::HashMap;

use anyhow::Result;
use itertools::Itertools;
use xml::{reader::XmlEvent, EventReader};

use crate::agent::Invocation;

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

    parsed
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

    // avoid running the same command twince in a row
    Ok(parsed.into_iter().unique().collect())
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
