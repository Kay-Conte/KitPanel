use serde::Serialize;

#[derive(Serialize)]
pub struct ServerStatus {
    pub online: bool,
}

#[derive(Serialize)]
pub struct GlobalStatus {
    pub servers: Vec<ServerStatus>,
}
