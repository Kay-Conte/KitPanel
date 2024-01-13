use models::{
    FromJson, GlobalStatus, InputCommandRequest, ServerOutput, ToJson, TokenRequest, TokenResponse,
};
use reqwest::{Client, Method};

use uuid::Uuid;

#[derive(Debug)]
pub enum Error {
    BadResponse,
    ParseError
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;

        match self {
            BadResponse => write!(f, "Bad response from server"),
            ParseError => write!(f, "Error parsing response from server"),
        }

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
                format!("http://{}/api/send_command/{}", self.address, server_id),
            )
            .body(InputCommandRequest { command }.to_json())
            .send()
            .await;

        match res {
            Ok(r) if r.status() == 200 => true,
            _ => false,
        }
    }

    pub async fn get_token(&self, username: String, password: String) -> Option<Uuid> {
        println!("Getting token");
        let res = reqwest::Client::new()
            .request(
                Method::GET,
                format!("http://{}/api/get_token/", self.address),
            )
            .body(TokenRequest { username, password }.to_json())
            .send()
            .await
            .ok()?;

        println!("Getting token");
        let body = res.text().await.ok()?;

        println!("Getting token");
        TokenResponse::from_json(body).and_then(|i| i.token)
    }
}
