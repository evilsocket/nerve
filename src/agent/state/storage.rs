use std::{ops::Deref, time::Instant /* , time::SystemTime*/};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::agent::events::{Event, EventType, Sender};

#[derive(Debug)]
pub struct Entry {
    pub time: Instant,
    pub complete: bool, // for Completion storage
    pub data: String,
}

impl Entry {
    pub fn new(data: String) -> Self {
        let time: Instant = Instant::now();
        let complete = false;
        Self {
            time,
            data,
            complete,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StorageType {
    // a single text entry
    Text,
    // a list indexed by element position
    Untagged,
    // a key=value store
    Tagged,
    // a single state with an optional previous state
    CurrentPrevious,
    // a list of tasks that can be set as complete
    Completion,
    // current time
    Time,
}

impl StorageType {
    pub fn as_u8(&self) -> u8 {
        match self {
            StorageType::Time => 0,
            StorageType::CurrentPrevious => 1,
            StorageType::Completion => 2,
            StorageType::Untagged => 3,
            StorageType::Tagged => 4,
            StorageType::Text => 5,
        }
    }
}

impl Default for StorageType {
    fn default() -> Self {
        Self::Text
    }
}

pub const CURRENT_TAG: &str = "__current";
pub const PREVIOUS_TAG: &str = "__previous";
pub const STARTED_AT_TAG: &str = "__started_at";
pub const TEXT_TAG: &str = "__text";

#[derive(Debug)]
pub struct Storage {
    events_tx: Sender,
    name: String,
    type_: StorageType,
    inner: IndexMap<String, Entry>,
}

impl Deref for Storage {
    type Target = IndexMap<String, Entry>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[allow(dead_code)]
impl Storage {
    pub fn new(name: &str, type_: StorageType, events_tx: Sender) -> Self {
        let name = name.to_string();
        let mut inner = IndexMap::new();

        if matches!(type_, StorageType::Time) {
            inner.insert(STARTED_AT_TAG.to_string(), Entry::new(String::new()));
        }

        Self {
            name,
            type_,
            inner,
            events_tx,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_type(&self) -> &StorageType {
        &self.type_
    }

    async fn on_event(&self, event: Event) -> anyhow::Result<()> {
        self.events_tx.send(event)?;
        Ok(())
    }

    pub fn get_started_at(&self) -> Instant {
        assert!(matches!(self.type_, StorageType::Time));
        self.inner.get(STARTED_AT_TAG).unwrap().time
    }

    pub fn get_text(&self) -> Option<String> {
        assert!(matches!(self.type_, StorageType::Text));
        self.inner.get(TEXT_TAG).map(|va| va.data.to_string())
    }

    pub async fn set_text(&mut self, data: &str) {
        assert!(matches!(self.type_, StorageType::Text));
        self.inner
            .insert(TEXT_TAG.to_string(), Entry::new(data.to_string()));
        self.on_event(Event::new(EventType::StorageUpdate {
            storage_name: self.name.to_string(),
            storage_type: self.type_,
            key: TEXT_TAG.to_string(),
            prev: None,
            new: Some(data.to_string()),
        }))
        .await
        .unwrap();
    }

    pub async fn add_text(&mut self, data: &str) {
        assert!(matches!(self.type_, StorageType::Text));

        let old = self.get_text();
        let new = old
            .as_ref()
            .map(|old| format!("{}\n{}", old, data))
            .unwrap_or(data.to_string());

        self.inner
            .insert(TEXT_TAG.to_string(), Entry::new(new.to_string()));

        self.on_event(Event::new(EventType::StorageUpdate {
            storage_name: self.name.to_string(),
            storage_type: self.type_,
            key: TEXT_TAG.to_string(),
            prev: old,
            new: Some(new),
        }))
        .await
        .unwrap();
    }

    pub async fn add_data(&mut self, key: &str, data: &str) {
        self.inner
            .insert(key.to_string(), Entry::new(data.to_string()));
        self.on_event(Event::new(EventType::StorageUpdate {
            storage_name: self.name.to_string(),
            storage_type: self.type_,
            key: key.to_string(),
            prev: None,
            new: Some(data.to_string()),
        }))
        .await
        .unwrap();
    }

    pub async fn add_tagged(&mut self, key: &str, data: &str) {
        assert!(matches!(self.type_, StorageType::Tagged));

        self.inner
            .insert(key.to_string(), Entry::new(data.to_string()));

        self.on_event(Event::new(EventType::StorageUpdate {
            storage_name: self.name.to_string(),
            storage_type: self.type_,
            key: key.to_string(),
            prev: None,
            new: Some(data.to_string()),
        }))
        .await
        .unwrap();
    }

    pub async fn del_tagged(&mut self, key: &str) -> Option<String> {
        assert!(matches!(self.type_, StorageType::Tagged));
        if let Some(old) = self.inner.shift_remove(key) {
            self.on_event(Event::new(EventType::StorageUpdate {
                storage_name: self.name.to_string(),
                storage_type: self.type_,
                key: key.to_string(),
                prev: Some(old.data.to_string()),
                new: None,
            }))
            .await
            .unwrap();

            Some(old.data)
        } else {
            None
        }
    }

    pub fn get_tagged(&self, key: &str) -> Option<String> {
        assert!(matches!(self.type_, StorageType::Tagged));
        self.inner.get(key).map(|va| va.data.to_string())
    }

    pub async fn add_completion(&mut self, data: &str) {
        assert!(matches!(self.type_, StorageType::Completion));
        let tag = format!("{}", self.inner.len() + 1);
        self.inner
            .insert(tag.to_string(), Entry::new(data.to_string()));

        self.on_event(Event::new(EventType::StorageUpdate {
            storage_name: self.name.to_string(),
            storage_type: self.type_,
            key: tag,
            prev: None,
            new: Some(data.to_string()),
        }))
        .await
        .unwrap();
    }

    pub async fn del_completion(&mut self, pos: usize) -> Option<String> {
        assert!(matches!(self.type_, StorageType::Completion));
        let tag = format!("{}", pos);
        if let Some(old) = self.inner.shift_remove(&tag) {
            self.on_event(Event::new(EventType::StorageUpdate {
                storage_name: self.name.to_string(),
                storage_type: self.type_,
                key: tag,
                prev: Some(old.data.to_string()),
                new: None,
            }))
            .await
            .unwrap();

            Some(old.data)
        } else {
            None
        }
    }

    pub async fn set_complete(&mut self, pos: usize) -> Option<bool> {
        assert!(matches!(self.type_, StorageType::Completion));
        let tag = format!("{}", pos);
        if let Some(entry) = self.inner.get_mut(&tag) {
            let prev = entry.complete;
            entry.complete = true;

            self.on_event(Event::new(EventType::StorageUpdate {
                storage_name: self.name.to_string(),
                storage_type: self.type_,
                key: tag,
                prev: Some((if prev { "complete" } else { "incomplete" }).to_string()),
                new: Some("complete".to_string()),
            }))
            .await
            .unwrap();

            Some(prev)
        } else {
            None
        }
    }

    pub async fn set_incomplete(&mut self, pos: usize) -> Option<bool> {
        assert!(matches!(self.type_, StorageType::Completion));
        let tag = format!("{}", pos);
        if let Some(entry) = self.inner.get_mut(&tag) {
            let prev = entry.complete;
            entry.complete = false;

            self.on_event(Event::new(EventType::StorageUpdate {
                storage_name: self.name.to_string(),
                storage_type: self.type_,
                key: tag,
                prev: Some((if prev { "complete" } else { "incomplete" }).to_string()),
                new: Some("incomplete".to_string()),
            }))
            .await
            .unwrap();

            Some(prev)
        } else {
            None
        }
    }

    pub async fn add_untagged(&mut self, data: &str) {
        assert!(matches!(self.type_, StorageType::Untagged));
        let tag = format!("{}", self.inner.len() + 1);
        self.inner
            .insert(tag.to_string(), Entry::new(data.to_string()));

        self.on_event(Event::new(EventType::StorageUpdate {
            storage_name: self.name.to_string(),
            storage_type: self.type_,
            key: tag,
            prev: None,
            new: Some(data.to_string()),
        }))
        .await
        .unwrap();
    }

    pub async fn del_untagged(&mut self, pos: usize) -> Option<String> {
        assert!(matches!(self.type_, StorageType::Untagged));
        let tag = format!("{}", pos);
        if let Some(old) = self.inner.shift_remove(&tag) {
            self.on_event(Event::new(EventType::StorageUpdate {
                storage_name: self.name.to_string(),
                storage_type: self.type_,
                key: tag,
                prev: Some(old.data.to_string()),
                new: None,
            }))
            .await
            .unwrap();

            Some(old.data)
        } else {
            None
        }
    }

    pub async fn set_current(&mut self, data: &str) {
        assert!(matches!(self.type_, StorageType::CurrentPrevious));

        let old_current = self.inner.shift_remove(CURRENT_TAG);
        self.inner
            .insert(CURRENT_TAG.to_string(), Entry::new(data.to_string()));
        let prev = if let Some(old_curr) = old_current {
            let data = old_curr.data.to_string();
            self.inner.insert(PREVIOUS_TAG.to_string(), old_curr);
            Some(data)
        } else {
            None
        };

        self.on_event(Event::new(EventType::StorageUpdate {
            storage_name: self.name.to_string(),
            storage_type: self.type_,
            key: CURRENT_TAG.to_string(),
            prev,
            new: Some(data.to_string()),
        }))
        .await
        .unwrap();
    }

    pub async fn clear(&mut self) {
        self.inner.clear();
        self.on_event(Event::new(EventType::StorageUpdate {
            storage_name: self.name.to_string(),
            storage_type: self.type_,
            key: "".to_string(),
            prev: None,
            new: None,
        }))
        .await
        .unwrap();
    }
}
