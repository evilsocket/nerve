use crate::agent::{
    namespaces::Action,
    state::storage::{Storage, StorageType, CURRENT_TAG, PREVIOUS_TAG},
    Invocation,
};

pub(crate) fn invocation(inv: &Invocation) -> String {
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
pub(crate) fn action(action: &Box<dyn Action>) -> String {
    let mut xml = format!("<{}", action.name());

    if let Some(attrs) = action.example_attributes() {
        for (name, example_value) in &attrs {
            xml += &format!(" {}=\"{}\"", name, example_value);
        }
    }

    if let Some(payload) = action.example_payload() {
        // TODO: escape payload?
        xml += &format!(">{}</{}>", payload, action.name());
    } else {
        xml += "/>";
    }

    xml
}

pub(crate) fn storage(storage: &Storage) -> String {
    if storage.is_empty() {
        return "".to_string();
    }

    match storage.get_type() {
        StorageType::Time => {
            let started_at = storage.get_started_at();

            format!(
                "## Current date: {}\n",
                chrono::Local::now().format("%m %B %Y %H:%M")
            ) + &format!("## Time since start: {:?}\n", started_at.elapsed())
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
