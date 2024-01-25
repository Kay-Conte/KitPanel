use serde::{Deserialize, Serialize};

use crate::fs::Config;

#[derive(Deserialize, Serialize)]
pub struct Settings {
    scale_factor: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            scale_factor: 0.8
        }
    }
}

impl Config for Settings {
    fn rel_path(rel: std::path::PathBuf) -> std::path::PathBuf {
        rel.join("settings.json")
    }

    fn bytes(&self) -> Vec<u8> {
        todo!()
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        todo!()
    }
}
