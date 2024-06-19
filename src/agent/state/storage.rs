use std::{sync::Mutex /* , time::SystemTime*/};

use colored::Colorize;
use indexmap::IndexMap;

// TODO: investigate other uses of IndexMap around the project

#[derive(Debug)]
pub(crate) struct Entry {
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
#[derive(Debug, Clone, Copy)]
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
    name: String,
    type_: StorageType,
    inner: Mutex<IndexMap<String, Entry>>,
}

#[allow(dead_code)]
impl Storage {
    pub fn new(name: &str, type_: StorageType) -> Self {
        let name = name.to_string();
        let inner = Mutex::new(IndexMap::new());
        Self { name, type_, inner }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_type(&self) -> &StorageType {
        &self.type_
    }

    pub fn get_inner(&self) -> &Mutex<IndexMap<String, Entry>> {
        &self.inner
    }

    pub fn add_tagged(&self, key: &str, data: &str) {
        assert!(matches!(self.type_, StorageType::Tagged));
        println!("<{}> {}={}", self.name.bold(), key, data.yellow());
        self.inner
            .lock()
            .unwrap()
            .insert(key.to_string(), Entry::new(data.to_string()));
    }

    pub fn del_tagged(&self, key: &str) -> Option<String> {
        assert!(matches!(self.type_, StorageType::Tagged));
        if let Some(old) = self.inner.lock().unwrap().shift_remove(key) {
            println!("<{}> {} removed\n", self.name.bold(), key);
            Some(old.data)
        } else {
            None
        }
    }

    pub fn get_tagged(&self, key: &str) -> Option<String> {
        assert!(matches!(self.type_, StorageType::Tagged));
        self.inner
            .lock()
            .unwrap()
            .get(key)
            .map(|va| va.data.to_string())
    }

    pub fn add_completion(&self, data: &str) {
        assert!(matches!(self.type_, StorageType::Completion));
        println!("<{}> {}", self.name.bold(), data.yellow());

        let mut inner = self.inner.lock().unwrap();

        let tag = format!("{}", inner.len() + 1);
        inner.insert(tag, Entry::new(data.to_string()));
    }

    pub fn del_completion(&self, pos: usize) -> Option<String> {
        assert!(matches!(self.type_, StorageType::Completion));
        let tag = format!("{}", pos);
        if let Some(old) = self.inner.lock().unwrap().shift_remove(&tag) {
            println!("<{}> element {} removed\n", self.name.bold(), pos);
            Some(old.data)
        } else {
            None
        }
    }

    pub fn set_complete(&self, pos: usize) -> Option<bool> {
        assert!(matches!(self.type_, StorageType::Completion));
        let tag = format!("{}", pos);
        if let Some(entry) = self.inner.lock().unwrap().get_mut(&tag) {
            println!("<{}> element {} set as complete\n", self.name.bold(), pos);
            let prev = entry.complete;
            entry.complete = true;
            Some(prev)
        } else {
            None
        }
    }

    pub fn set_incomplete(&self, pos: usize) -> Option<bool> {
        assert!(matches!(self.type_, StorageType::Completion));
        let tag = format!("{}", pos);
        if let Some(entry) = self.inner.lock().unwrap().get_mut(&tag) {
            println!("<{}> element {} set as incomplete\n", self.name.bold(), pos);
            let prev = entry.complete;
            entry.complete = false;
            Some(prev)
        } else {
            None
        }
    }

    pub fn add_untagged(&self, data: &str) {
        assert!(matches!(self.type_, StorageType::Untagged));
        println!("<{}> {}", self.name.bold(), data.yellow());

        let mut inner = self.inner.lock().unwrap();

        let tag = format!("{}", inner.len() + 1);
        inner.insert(tag, Entry::new(data.to_string()));
    }

    pub fn del_untagged(&self, pos: usize) -> Option<String> {
        assert!(matches!(self.type_, StorageType::Untagged));
        let tag = format!("{}", pos);
        if let Some(old) = self.inner.lock().unwrap().shift_remove(&tag) {
            println!("<{}> element {} removed\n", self.name.bold(), pos);
            Some(old.data)
        } else {
            None
        }
    }

    pub fn set_current(&self, data: &str, verbose: bool) {
        assert!(matches!(self.type_, StorageType::CurrentPrevious));
        let mut inner = self.inner.lock().unwrap();

        if verbose {
            println!("<{}> current={}", self.name.bold(), data.yellow());
        }

        let old_current = inner.shift_remove(CURRENT_TAG);

        inner.insert(CURRENT_TAG.to_string(), Entry::new(data.to_string()));
        if let Some(old_curr) = old_current {
            inner.insert(PREVIOUS_TAG.to_string(), old_curr);
        }
    }

    pub fn clear(&self) {
        self.inner.lock().unwrap().clear();
        println!("<{}> cleared", self.name.bold());
    }
}
