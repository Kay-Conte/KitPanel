use std::{path::PathBuf, sync::{Arc, RwLock}, fs::File, io::{Read, Write}};

use foxhole::type_cache::TypeCacheKey;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerInfo {
    pub id: String,
    pub start_command: String,
}

impl ServerInfo {
    fn template() -> Self {
        Self {
            id: "Example".to_string(),
            start_command: "ExampleStartCommand".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Configuration {
    pub base_directory: PathBuf,
    pub servers: Vec<ServerInfo>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            base_directory: Configuration::config_path(),
            servers: vec![ServerInfo::template()],
        }
    }
}

impl TypeCacheKey for Configuration {
    type Value = Arc<RwLock<Configuration>>;
}

impl Configuration {
    fn config_path() -> PathBuf {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("config.json")
    }

    pub fn get_or_create() -> Self {
        match File::open(Configuration::config_path()) {
            Ok(mut file) => {
                let mut json = String::new();

                let _ = file.read_to_string(&mut json);

                serde_json::from_str(&json).expect("invalid configuration file")
            }

            Err(_) => {
                let config = Configuration::default();

                config.save();

                config
            }
        }
    }

    pub fn save(&self) {
        let data = serde_json::to_string_pretty(self).unwrap();

        let mut file = File::create(Configuration::config_path()).unwrap();

        let _ = file.write_all(data.as_bytes());
    }
}
