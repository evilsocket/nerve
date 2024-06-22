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

pub(crate) fn storage(storage: &Storage) -> String {
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