use anyhow::Result;

use super::{namespaces::NAMESPACES, state::State};

pub(crate) mod xml;

pub(crate) fn available_actions() -> String {
    let mut md = "".to_string();

    for build_fn in NAMESPACES.values() {
        let group = build_fn();
        md += &format!("## {}\n\n", group.name);
        if !group.description.is_empty() {
            md += &format!("{}\n\n", group.description);
        }
        for action in &group.actions {
            md += &format!(
                "{} {}\n\n",
                action.description(),
                self::xml::serialize::action(action)
            );
        }
    }

    md.trim().to_string()
}

fn state_available_actions(state: &State) -> Result<String> {
    let mut md = "".to_string();

    for group in state.get_namespaces() {
        md += &format!("## {}\n\n", group.name);
        if !group.description.is_empty() {
            md += &format!("{}\n\n", group.description);
        }
        for action in &group.actions {
            md += &format!(
                "{} {}\n\n",
                action.description(),
                self::xml::serialize::action(action)
            );
        }
    }

    Ok(md)
}

pub(crate) fn state_to_system_prompt(state: &State) -> Result<String> {
    let task = state.get_task();
    let system_prompt = task.to_system_prompt()?;

    let mut storages = vec![];
    let mut sorted = state.get_storages();
    sorted.sort_by_key(|x| x.get_type().as_u8());

    for storage in sorted {
        storages.push(self::xml::serialize::storage(storage));
    }

    let storages = storages.join("\n\n");
    let guidance = task
        .guidance()?
        .into_iter()
        .map(|s| format!("- {}", s))
        .collect::<Vec<String>>()
        .join("\n");
    let available_actions = state_available_actions(state)?;

    let iterations = if state.metrics.max_steps > 0 {
        format!(
            "You are currently at step {} of a maximum of {}.",
            state.metrics.current_step + 1,
            state.metrics.max_steps
        )
    } else {
        "".to_string()
    };

    Ok(format!(
        include_str!("system.prompt"),
        iterations = iterations,
        system_prompt = system_prompt,
        storages = storages,
        available_actions = available_actions,
        guidance = guidance,
    ))
}
