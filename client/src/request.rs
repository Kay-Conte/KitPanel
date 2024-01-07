use models::{FromJson, GlobalStatus};
use reqwest::{Client, Method};

#[derive(Debug)]
pub enum Error {

}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

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

    async fn get_status(&self) -> reqwest::Result<GlobalStatus> {
        let res = reqwest::Client::new()
            .request(Method::GET, "http://127.0.0.1:8080/api/status")
            .send()
            .await?;

        let body = res.text().await?;

        Ok(GlobalStatus::from_json(&body).unwrap())
    }
}

pub async fn get_status() -> Option<GlobalStatus> {
    let res = reqwest::Client::new()
        .request(Method::GET, "http://127.0.0.1:8080/api/status")
        .send()
        .await
        .ok()?;

    let body = res.text().await.ok()?;

    GlobalStatus::from_json(&body)
}

pub async fn start_server(server_id: String) -> bool {
    let res = reqwest::Client::new()
        .request(
            Method::POST,
            format!("http://127.0.0.1:8080/api/start/{}", server_id),
        )
        .send()
        .await;

    res.unwrap().status() == 200
}

pub async fn stop_server(server_id: String) -> bool {
    let res = reqwest::Client::new()
        .request(
            Method::POST,
            format!("http://127.0.0.1:8080/api/start/{}", server_id),
        )
        .send()
        .await;

    res.unwrap().status() == 200
}
