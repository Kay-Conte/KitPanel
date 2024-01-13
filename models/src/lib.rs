use serde::{Serialize, Deserialize, de::DeserializeOwned};
use uuid::Uuid;

pub trait ToJson {
    fn to_json(&self) -> String;
}

impl<T> ToJson for T where T: Serialize {
    fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize object")
    }
}

pub trait FromJson: Sized {
    fn from_json(from: String) -> Option<Self>;
}

impl <T> FromJson for T where T: DeserializeOwned {
    fn from_json(from: String) -> Option<Self> {
        serde_json::from_str(&from).ok()
    }
}


#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerStatus {
    pub id: String,
    pub running: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GlobalStatus {
    pub servers: Vec<ServerStatus>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ServerOutput {
    pub output: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct InputCommandRequest {
    pub command: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TokenRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TokenResponse {
    pub token: Option<Uuid>,
}
