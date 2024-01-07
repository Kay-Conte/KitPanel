mod configuration;
mod json;

use std::{
    collections::HashMap,
    process::{Child, Command},
    sync::{Arc, RwLock},
};

use configuration::Configuration;
use foxhole::{
    framework::run_with_cache,
    sys,
    systems::{Query, UrlPart},
    type_cache::{TypeCache, TypeCacheKey},
    Get, Post, Route,
};
use json::Json;
use models::{GlobalStatus, ServerStatus};

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

fn status(
    _g: Get,
    Query(config): Query<Configuration>,
    Query(running): Query<Running>,
) -> Json<GlobalStatus> {
    let running = &running.read().unwrap().0;
    let config = config.read().unwrap();

    let servers = config
        .servers
        .iter()
        .map(|info| ServerStatus {
            id: info.id.clone(),
            running: running.contains_key(&info.id),
        })
        .collect();

    Json(GlobalStatus { servers })
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

fn stop(_p: Post, UrlPart(server_id): UrlPart, Query(running): Query<Running>) -> u16 {
    let mut running = running.write().unwrap();

    match running.0.get_mut(&server_id) {
        Some(process) => {
            if process.child.kill().is_err() {
                return 500;
            }
        }
        None => {}
    }

    200
}

fn main() {
    let router = Route::empty().route("web", sys![]).route(
        "api",
        Route::empty()
            .route("status", sys![status])
            .route("start", sys![start])
            .route("stop", sys![stop]),
    );

    println!("Server is running on '0.0.0.0:8080'");

    let mut cache = TypeCache::new();

    cache.insert::<Configuration>(Arc::new(RwLock::new(Configuration::get_or_create())));
    cache.insert::<Running>(Arc::new(RwLock::new(Running::default())));

    run_with_cache("0.0.0.0:8080", router, cache);
}
