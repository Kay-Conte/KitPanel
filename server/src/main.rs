mod configuration;
mod json;

use std::{
    collections::HashMap,
    process::{Child, Command},
    sync::{Arc, RwLock},
};

use configuration::{Configuration, ServerInfo};
use foxhole::{
    framework::run_with_cache,
    sys,
    systems::{Query, UrlPart},
    type_cache::{TypeCache, TypeCacheKey},
    Get, Post, Route,
};
use json::Json;
use models::GlobalStatus;

pub struct Process {
    child: Child,
}

impl Process {
    fn new(child: Child) -> Self {
        Self { child }
    }
}

#[derive(Default)]
pub struct Running(HashMap<String, Process>);

impl TypeCacheKey for Running {
    type Value = Arc<RwLock<Running>>;
}

fn status(_g: Get, Query(running): Query<Running>) -> Json<GlobalStatus> {
    let running = running
        .read()
        .unwrap()
        .0
        .iter()
        .map(|i| models::ServerInfo { id: i.0.clone() })
        .collect();

    Json(GlobalStatus { running })
}

fn start(
    _p: Post,
    UrlPart(server_id): UrlPart,
    Query(config): Query<Configuration>,
    Query(running): Query<Running>,
) -> u16 {
    {
        let running = running.read().unwrap();

        if running.0.contains_key(&server_id) {
            return 200;
        }
    }

    let config = config.read().unwrap().clone();

    let Some(server) = config.servers.iter().find(|i| i.id == server_id) else {
        return 404;
    };

    let dir = config.base_directory.join(server.id.clone());

    if std::fs::create_dir_all(&dir).is_err() {
        return 500;
    };

    let mut iter = server.start_command.split_whitespace();

    let Some(first) = iter.next() else {
        return 500;
    };

    let Ok(process) = Command::new(first).args(iter).current_dir(dir).spawn() else {
        return 500;
    };

    {
        let mut running = running.write().unwrap();

        running.0.insert(server.id.clone(), Process::new(process));
    }

    200
}

fn main() {
    let router = Route::empty().route("web", sys![]).route(
        "api",
        Route::empty()
            .route("status", sys![status])
            .route("start", sys![start]),
    );

    println!("Server is running on '0.0.0.0:8080'");

    let mut cache = TypeCache::new();

    cache.insert::<Configuration>(Arc::new(RwLock::new(Configuration::get_or_create())));
    cache.insert::<Running>(Arc::new(RwLock::new(Running::default())));

    run_with_cache("0.0.0.0:8080", router, cache);
}
