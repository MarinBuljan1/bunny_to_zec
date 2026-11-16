use crate::persistence::{StorageError, Store};
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct MemoryStore {
    pub values: HashMap<String, String>,
    pub read_error: Option<StorageError>,
    pub write_error: Option<StorageError>,
    pub remove_error: Option<StorageError>,
    pub writes: usize,
}

impl MemoryStore {
    pub fn insert_json(&mut self, key: &str, value: &impl serde::Serialize) {
        self.values
            .insert(key.into(), serde_json::to_string(value).unwrap());
    }
}

impl Store for MemoryStore {
    fn get(&self, key: &str) -> Result<Option<String>, StorageError> {
        if let Some(error) = &self.read_error {
            return Err(error.clone());
        }
        Ok(self.values.get(key).cloned())
    }
    fn set(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
        if let Some(error) = &self.write_error {
            return Err(error.clone());
        }
        self.writes += 1;
        self.values.insert(key.into(), value.into());
        Ok(())
    }

    fn remove(&mut self, key: &str) -> Result<(), StorageError> {
        if let Some(error) = &self.remove_error {
            return Err(error.clone());
        }
        self.values.remove(key);
        Ok(())
    }
}

pub(crate) fn content() -> crate::model::LearningContent {
    serde_json::from_str(
        r#"{
        "cards":{"bok":{"base_language":"hello"}},
        "explanations":{}
    }"#,
    )
    .unwrap()
}
