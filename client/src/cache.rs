use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Default, Debug)]
pub struct Cache {
    pub last_address: String,
    pub last_username: String,
}

impl Cache {
    pub fn new(last_ip: String, last_username: String) -> Self {
        Self {
            last_address: last_ip,
            last_username
        } 
    }
}

impl crate::fs::Config for Cache {
    fn rel_path(rel: std::path::PathBuf) -> std::path::PathBuf {
        rel.join("cache.json")
    }

    fn bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("Failed to parse to json")
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()
    }
}
