use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
};

use foxhole::type_cache::TypeCacheKey;
use serde::{Deserialize, Serialize};

use crate::fs::Config;

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerInfo {
    pub id: String,
    pub display: String,
    pub start_command: String,
}

impl ServerInfo {
    fn template() -> Self {
        Self {
            id: "example".to_string(),
            display: "Example".to_string(),
            start_command: "example start command".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerConfig {
    pub address: String,
    pub port: String,
    pub server_directory: PathBuf,
    pub servers: Vec<ServerInfo>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "0.0.0.0".to_string(),
            port: "8080".to_string(),
            server_directory: ServerConfig::server_dir(),
            servers: vec![ServerInfo::template()],
        }
    }
}

impl Config for ServerConfig {
    fn rel_path(rel: PathBuf) -> PathBuf {
        rel.join("servers.json")
    }

    fn bytes(&self) -> Vec<u8> {
        serde_json::to_vec_pretty(self).unwrap()
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()
    }
}

impl TypeCacheKey for ServerConfig {
    type Value = Arc<RwLock<ServerConfig>>;
}

impl ServerConfig {
    fn server_dir() -> PathBuf {
        Self::full_path().parent().unwrap().to_path_buf().join("servers")
    }
}
