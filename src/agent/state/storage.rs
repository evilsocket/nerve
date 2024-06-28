use std::{ops::Deref /* , time::SystemTime*/};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::agent::events::{Event, Sender};

#[derive(Debug)]
pub struct Entry {
    //pub time: SystemTime,
    pub complete: bool, // for Completion storage
    pub data: String,
}

impl Entry {
    pub fn new(data: String) -> Self {
        //let time = SystemTime::now();
        let complete = false;
        Self {
            /* time ,*/ data,
            complete,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StorageType {
    // a list indexed by element position
    Untagged,
    // a key=value store
    Tagged,
    // a single state with an optional previous state
    CurrentPrevious,
    // a list of tasks that can be set as complete
    Completion,
}

impl StorageType {
    pub fn as_u8(&self) -> u8 {
        match self {
            StorageType::CurrentPrevious => 0,
            StorageType::Completion => 1,
            StorageType::Untagged => 2,
            StorageType::Tagged => 3,
        }
    }
}

pub(crate) const CURRENT_TAG: &str = "__current";
pub(crate) const PREVIOUS_TAG: &str = "__previous";

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
        let inner = IndexMap::new();
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

    fn on_event(&self, event: Event) {
        self.events_tx.send(event).unwrap();
    }

    pub fn add_tagged(&mut self, key: &str, data: &str) {
        assert!(matches!(self.type_, StorageType::Tagged));

        self.inner
            .insert(key.to_string(), Entry::new(data.to_string()));

        self.on_event(Event::StorageUpdate {
            storage_name: self.name.to_string(),
            storage_type: self.type_,
            key: key.to_string(),
            prev: None,
            new: Some(data.to_string()),
        });
    }

    pub fn del_tagged(&mut self, key: &str) -> Option<String> {
        assert!(matches!(self.type_, StorageType::Tagged));
        if let Some(old) = self.inner.shift_remove(key) {
            self.on_event(Event::StorageUpdate {
                storage_name: self.name.to_string(),
                storage_type: self.type_,
                key: key.to_string(),
                prev: Some(old.data.to_string()),
                new: None,
            });

            Some(old.data)
        } else {
            None
        }
    }

    pub fn get_tagged(&self, key: &str) -> Option<String> {
        assert!(matches!(self.type_, StorageType::Tagged));
        self.inner.get(key).map(|va| va.data.to_string())
    }

    pub fn add_completion(&mut self, data: &str) {
        assert!(matches!(self.type_, StorageType::Completion));
        let tag = format!("{}", self.inner.len() + 1);
        self.inner
            .insert(tag.to_string(), Entry::new(data.to_string()));

        self.on_event(Event::StorageUpdate {
            storage_name: self.name.to_string(),
            storage_type: self.type_,
            key: tag,
            prev: None,
            new: Some(data.to_string()),
        });
    }

    pub fn del_completion(&mut self, pos: usize) -> Option<String> {
        assert!(matches!(self.type_, StorageType::Completion));
        let tag = format!("{}", pos);
        if let Some(old) = self.inner.shift_remove(&tag) {
            self.on_event(Event::StorageUpdate {
                storage_name: self.name.to_string(),
                storage_type: self.type_,
                key: tag,
                prev: Some(old.data.to_string()),
                new: None,
            });

            Some(old.data)
        } else {
            None
        }
    }

    pub fn set_complete(&mut self, pos: usize) -> Option<bool> {
        assert!(matches!(self.type_, StorageType::Completion));
        let tag = format!("{}", pos);
        if let Some(entry) = self.inner.get_mut(&tag) {
            let prev = entry.complete;
            entry.complete = true;

            self.on_event(Event::StorageUpdate {
                storage_name: self.name.to_string(),
                storage_type: self.type_,
                key: tag,
                prev: Some((if prev { "complete" } else { "incomplete" }).to_string()),
                new: Some("complete".to_string()),
            });

            Some(prev)
        } else {
            None
        }
    }

    pub fn set_incomplete(&mut self, pos: usize) -> Option<bool> {
        assert!(matches!(self.type_, StorageType::Completion));
        let tag = format!("{}", pos);
        if let Some(entry) = self.inner.get_mut(&tag) {
            let prev = entry.complete;
            entry.complete = false;

            self.on_event(Event::StorageUpdate {
                storage_name: self.name.to_string(),
                storage_type: self.type_,
                key: tag,
                prev: Some((if prev { "complete" } else { "incomplete" }).to_string()),
                new: Some("incomplete".to_string()),
            });

            Some(prev)
        } else {
            None
        }
    }

    pub fn add_untagged(&mut self, data: &str) {
        assert!(matches!(self.type_, StorageType::Untagged));
        let tag = format!("{}", self.inner.len() + 1);
        self.inner
            .insert(tag.to_string(), Entry::new(data.to_string()));

        self.on_event(Event::StorageUpdate {
            storage_name: self.name.to_string(),
            storage_type: self.type_,
            key: tag,
            prev: None,
            new: Some(data.to_string()),
        });
    }

    pub fn del_untagged(&mut self, pos: usize) -> Option<String> {
        assert!(matches!(self.type_, StorageType::Untagged));
        let tag = format!("{}", pos);
        if let Some(old) = self.inner.shift_remove(&tag) {
            self.on_event(Event::StorageUpdate {
                storage_name: self.name.to_string(),
                storage_type: self.type_,
                key: tag,
                prev: Some(old.data.to_string()),
                new: None,
            });
            Some(old.data)
        } else {
            None
        }
    }

    pub fn set_current(&mut self, data: &str) {
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

        self.on_event(Event::StorageUpdate {
            storage_name: self.name.to_string(),
            storage_type: self.type_,
            key: CURRENT_TAG.to_string(),
            prev,
            new: Some(data.to_string()),
        });
    }

    pub fn clear(&mut self) {
        self.inner.clear();
        self.on_event(Event::StorageUpdate {
            storage_name: self.name.to_string(),
            storage_type: self.type_,
            key: "".to_string(),
            prev: None,
            new: None,
        });
    }
}
