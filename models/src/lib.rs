use serde::{Serialize, Deserialize};

pub trait ToJson {
    fn to_json(&self) -> String;
}

impl<T> ToJson for T where T: Serialize {
    fn to_json(&self) -> String {
        serde_json::to_string(self).expect("Failed to serialize object")
    }
}

pub trait FromJson<'a>: Sized {
    fn from_json(from: &'a str) -> Option<Self>;
}

impl <'a, T> FromJson<'a> for T where T: Deserialize<'a> {
    fn from_json(from: &'a str) -> Option<Self> {
        serde_json::from_str(from).ok()
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
