use models::{
    FromJson, GlobalStatus, InputCommandRequest, ServerOutput, ToJson, TokenRequest, TokenResponse,
};
use reqwest::{Client, Method};

use uuid::Uuid;

#[derive(Debug)]
pub enum Error {
    BadResponse,
    ParseError,
    NotAuthorized,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;

        match self {
            BadResponse => write!(f, "Bad response from server"),
            ParseError => write!(f, "Error parsing response from server"),
            NotAuthorized => write!(f, "Not authorized"),
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

    pub async fn get_status(&self, token: Uuid) -> Option<GlobalStatus> {
        let token = serde_json::to_string(&token).unwrap();

        let res = self
            .client
            .request(
                Method::GET,
                format!("http://{}/api/get_status", self.address),
            )
            .header("authorization", token)
            .send()
            .await
            .ok()?;

        let body = res.text().await.ok()?;

        GlobalStatus::from_json(body)
    }

    pub async fn get_output(&self, server_id: String, token: Uuid) -> Option<ServerOutput> {
        let token = serde_json::to_string(&token).unwrap();

        let res = self
            .client
            .request(
                Method::GET,
                format!("http://{}/api/get_output/{}", self.address, server_id),
            )
            .header("authorization", token)
            .send()
            .await
            .ok()?;

        let body = res.text().await.ok()?;

        ServerOutput::from_json(body)
    }

    pub async fn start_server(&self, server_id: String, token: Uuid) -> bool {
        let token = serde_json::to_string(&token).unwrap();

        let res = self
            .client
            .request(
                Method::POST,
                format!("http://{}/api/start/{}", self.address, server_id),
            )
            .header("authorization", token)
            .send()
            .await;

        match res {
            Ok(r) if r.status() == 200 => true,
            _ => false,
        }
    }

    pub async fn stop_server(&self, server_id: String, token: Uuid) -> bool {
        let token = serde_json::to_string(&token).unwrap();

        let res = self
            .client
            .request(
                Method::POST,
                format!("http://{}/api/stop/{}", self.address, server_id),
            )
            .header("authorization", token)
            .send()
            .await;

        match res {
            Ok(r) if r.status() == 200 => true,
            _ => false,
        }
    }

    pub async fn send_command(&self, server_id: String, command: String, token: Uuid) -> bool {
        let token = serde_json::to_string(&token).unwrap();

        let res = reqwest::Client::new()
            .request(
                Method::POST,
                format!("http://{}/api/send_command/{}", self.address, server_id),
            )
            .header("authorization", token)
            .body(InputCommandRequest { command }.to_json())
            .send()
            .await;

        match res {
            Ok(r) if r.status() == 200 => true,
            _ => false,
        }
    }

    pub async fn get_token(&self, username: String, password: String) -> Option<Uuid> {
        let res = reqwest::Client::new()
            .request(
                Method::GET,
                format!("http://{}/api/get_token/", self.address),
            )
            .body(TokenRequest { username, password }.to_json())
            .send()
            .await
            .ok()?;

        let body = res.text().await.ok()?;

        TokenResponse::from_json(body).and_then(|i| i.token)
    }
}
