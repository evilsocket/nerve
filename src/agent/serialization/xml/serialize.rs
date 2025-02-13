use crate::agent::{
    namespaces::Tool,
    state::storage::{Storage, StorageType, CURRENT_TAG, PREVIOUS_TAG},
    ToolCall,
};

pub fn tool_call(tool_call: &ToolCall) -> String {
    let mut xml = format!("<{}", tool_call.tool_name);
    if let Some(attrs) = &tool_call.named_arguments {
        for (key, value) in attrs {
            xml += &format!(" {key}=\"{value}\"");
        }
    }
    xml += &format!(
        ">{}</{}>",
        if let Some(data) = tool_call.argument.as_ref() {
            data
        } else {
            ""
        },
        tool_call.tool_name
    );

    xml
}

#[allow(clippy::borrowed_box)]
pub fn tool(tool: &Box<dyn Tool>) -> String {
    let mut xml = format!("`<{}", tool.name());

    if let Some(attrs) = tool.example_attributes() {
        for (name, example_value) in &attrs {
            xml += &format!(" {}=\"{}\"", name, example_value);
        }
    }

    if let Some(payload) = tool.example_payload() {
        // TODO: escape payload?
        xml += &format!(">{}</{}>`", payload, tool.name());
    } else {
        xml += "/>`";
    }

    xml
}

pub fn storage(storage: &Storage) -> String {
    if storage.is_empty() {
        return "".to_string();
    }

    match storage.get_type() {
        StorageType::Text => {
            if let Some(text) = storage.get_text() {
                format!("## Reasoning\n\n{}\n", text)
            } else {
                "".to_string()
            }
        }
        StorageType::Time => {
            let started_at = storage.get_started_at();

            let mut raw = format!(
                "## Current date: {}\n",
                chrono::Local::now().format("%m %B %Y %H:%M")
            );

            raw.push_str(&format!(
                "## Time since start: {:?}\n",
                started_at.elapsed()
            ));

            raw
        }
        StorageType::Tagged => {
            let mut xml: String = format!("<{}>\n", storage.get_name());

            for (key, entry) in storage.iter() {
                xml += &format!("  - {}={}\n", key, &entry.data);
            }

            xml += &format!("</{}>", storage.get_name());

            xml.to_string()
        }
        StorageType::Untagged => {
            let mut xml = format!("<{}>\n", storage.get_name());

            for entry in storage.values() {
                xml += &format!("  - {}\n", &entry.data);
            }

            xml += &format!("</{}>", storage.get_name());

            xml.to_string()
        }
        StorageType::Completion => {
            let mut xml = format!("<{}>\n", storage.get_name());

            for entry in storage.values() {
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
            if let Some(current) = storage.get(CURRENT_TAG) {
                let mut str = format!("* Current {}: {}", storage.get_name(), current.data.trim());
                if let Some(prev) = storage.get(PREVIOUS_TAG) {
                    str += &format!("\n* Previous {}: {}", storage.get_name(), prev.data.trim());
                }
                str
            } else {
                "".to_string()
            }
        }
    }
}
