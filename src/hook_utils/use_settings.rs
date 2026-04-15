use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type SettingsValue = serde_json::Value;

pub struct Settings {
    values: Arc<RwLock<HashMap<String, SettingsValue>>>,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            values: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<SettingsValue> {
        let values = self.values.read().await;
        values.get(key).cloned()
    }

    pub async fn set(&self, key: String, value: SettingsValue) {
        let mut values = self.values.write().await;
        values.insert(key, value);
    }

    pub async fn remove(&self, key: &str) -> Option<SettingsValue> {
        let mut values = self.values.write().await;
        values.remove(key)
    }

    pub async fn clear(&self) {
        let mut values = self.values.write().await;
        values.clear();
    }

    pub async fn keys(&self) -> Vec<String> {
        let values = self.values.read().await;
        values.keys().cloned().collect()
    }

    pub async fn len(&self) -> usize {
        let values = self.values.read().await;
        values.len()
    }

    pub async fn is_empty(&self) -> bool {
        let values = self.values.read().await;
        values.is_empty()
    }

    pub async fn get_all(&self) -> HashMap<String, SettingsValue> {
        let values = self.values.read().await;
        values.clone()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_settings() {
        let settings = Settings::new();

        settings
            .set("key1".to_string(), serde_json::json!("value1"))
            .await;

        let value = settings.get("key1").await;
        assert!(value.is_some());

        let keys = settings.keys().await;
        assert_eq!(keys.len(), 1);
    }
}
