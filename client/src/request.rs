use models::{FromJson, GlobalStatus};
use reqwest::Method;

pub async fn get_status() -> Option<GlobalStatus> {
    let res = reqwest::Client::new()
        .request(Method::GET, "http://127.0.0.1:8080/api/status")
        .send()
        .await
        .ok()?;

    println!("Successfully retrieved packet: {:?}", res.status());

    let body = res.text().await.ok()?;


    GlobalStatus::from_json(&body)
}

pub async fn start_server(server_id: String) -> bool {
    println!("starting server");

    let res = reqwest::Client::new()
        .request(
            Method::POST,
            format!("http://127.0.0.1:8080/api/start/{}", server_id),
        )
        .send()
        .await;
    
    println!("Server started {}", res.unwrap().status());
    
    true
}
