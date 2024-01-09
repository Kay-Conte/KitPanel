use models::{FromJson, GlobalStatus, InputCommandRequest, ServerOutput, ToJson};
use reqwest::{Client, Method};

#[derive(Debug)]
pub enum Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

#[derive(Debug, Clone)]
pub struct Request {
    client: Client,
    address: String,
}

impl Request {
    pub fn new(address: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            address,
        }
    }

    pub async fn get_status(&self) -> Option<GlobalStatus> {
        let res = self
            .client
            .request(
                Method::GET,
                format!("http://{}/api/get_status", self.address),
            )
            .send()
            .await
            .ok()?;

        let body = res.text().await.ok()?;

        GlobalStatus::from_json(body)
    }

    pub async fn get_output(&self, server_id: String) -> Option<ServerOutput> {
        let res = self
            .client
            .request(
                Method::GET,
                format!("http://{}/api/get_output/{}", self.address, server_id),
            )
            .send()
            .await
            .ok()?;

        let body = res.text().await.ok()?;

        ServerOutput::from_json(body)
    }

    pub async fn start_server(&self, server_id: String) -> bool {
        let res = self
            .client
            .request(
                Method::POST,
                format!("http://{}/api/start/{}", self.address, server_id),
            )
            .send()
            .await;

        match res {
            Ok(r) if r.status() == 200 => true,
            _ => false,
        }
    }

    pub async fn stop_server(&self, server_id: String) -> bool {
        let res = self
            .client
            .request(
                Method::POST,
                format!("http://{}/api/stop/{}", self.address, server_id),
            )
            .send()
            .await;

        match res {
            Ok(r) if r.status() == 200 => true,
            _ => false,
        }
    }

    pub async fn send_command(&self, server_id: String, command: String) -> bool {
        let res = reqwest::Client::new()
            .request(
                Method::POST,
                format!("http://127.0.0.1:8080/api/send_command/{}", server_id),
            )
            .body(InputCommandRequest { command }.to_json())
            .send()
            .await;

        match res {
            Ok(r) if r.status() == 200 => true,
            _ => false,
        }
    }
}
