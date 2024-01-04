use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ServerStatus {
    pub online: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GlobalStatus {
    pub servers: Vec<ServerStatus>,
}
