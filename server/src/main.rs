mod json;

use foxhole::{run, sys, systems::Endpoint, Get, Route};
use json::Json;
use models::response::GlobalStatus;

fn redirect(_g: Get, _e: Endpoint) {

}

fn status(_g: Get) -> Json<GlobalStatus> {
    Json(GlobalStatus {
        servers: Vec::new(),
    })
}

fn main() {
    let router = Route::new(sys![redirect])
        .route("web", sys![])
        .route("api", Route::empty().route("status", sys![status]));
    
    println!("Server is running on '127.0.0.1:8080'");

    run("127.0.0.1:8080", router);
}
