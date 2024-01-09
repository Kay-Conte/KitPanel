mod configuration;
mod json;
mod process;

use std::{
    collections::HashMap,
    process::{Command, Stdio},
    sync::{Arc, RwLock},
};

use configuration::Configuration;
use foxhole::{
    framework::run_with_cache,
    sys,
    type_cache::{TypeCache, TypeCacheKey},
    Route, resolve::{Get, Query, Post, UrlPart},
};
use json::Json;
use models::{GlobalStatus, InputCommandRequest, ServerOutput, ServerStatus};
use process::Process;

#[derive(Default)]
pub struct ProcessCache(HashMap<String, Process>);

impl TypeCacheKey for ProcessCache {
    type Value = Arc<RwLock<ProcessCache>>;
}

fn get_status(
    _g: Get,
    Query(config): Query<Configuration>,
    Query(running): Query<ProcessCache>,
) -> Json<GlobalStatus> {
    let running = &running.read().unwrap().0;
    let config = config.read().unwrap();

    let servers = config
        .servers
        .iter()
        .map(|info| {
            let running = running.get(&info.id).map(|p| p.is_alive()).unwrap_or(false);

            ServerStatus {
                id: info.id.clone(),
                running,
            }
        })
        .collect();

    Json(GlobalStatus { servers })
}

fn start(
    _p: Post,
    UrlPart(server_id): UrlPart,
    Query(config): Query<Configuration>,
    Query(running): Query<ProcessCache>,
) -> u16 {
    {
        let running = running.read().unwrap();

        let running = running
            .0
            .get(&server_id)
            .map(|p| p.is_alive())
            .unwrap_or(false);

        if running {
            return 200;
        }
    }

    let config = config.read().unwrap().clone();

    let Some(server) = config.servers.iter().find(|i| i.id == server_id) else {
        return 404;
    };

    let dir = config
        .base_directory
        .join("servers")
        .join(server.id.clone());

    if std::fs::create_dir_all(&dir).is_err() {
        return 500;
    };

    let mut iter = server.start_command.split_whitespace();

    let Some(first) = iter.next() else {
        return 500;
    };

    let Ok(child) = Command::new(first)
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .args(iter)
        .current_dir(dir)
        .spawn()
    else {
        return 500;
    };

    {
        let mut running = running.write().unwrap();

        if let Some(process) = running.0.get_mut(&server.id) {
            process.insert(child)
        } else {
            running.0.insert(server.id.clone(), Process::new(child));
        }
    }

    200
}

fn stop(_p: Post, UrlPart(server_id): UrlPart, Query(running): Query<ProcessCache>) -> u16 {
    let mut running = running.write().unwrap();

    match running.0.get_mut(&server_id) {
        Some(process) => {
            if process.kill().is_err() {
                return 500;
            }
        }
        None => {}
    }

    200
}

fn get_output(
    _g: Get,
    UrlPart(server_id): UrlPart,
    Query(running): Query<ProcessCache>,
) -> Json<ServerOutput> {
    let running = running.read().unwrap();

    let Some(server) = running.0.get(&server_id) else {
        return Json(ServerOutput { output: None });
    };

    let output = server.console.inner();

    Json(ServerOutput {
        output: Some(output),
    })
}

fn send_command(
    _p: Post,
    UrlPart(server_id): UrlPart,
    Json(command): Json<InputCommandRequest>,
    Query(processes): Query<ProcessCache>,
) -> u16 {
    let mut processes = processes.write().unwrap();

    let Some(process) = processes.0.get_mut(&server_id) else {
        return 200;
    };

    process.send(command.command);

    200
}

fn main() {
    let router = Route::empty().route("web", sys![]).route(
        "api",
        Route::empty()
            .route("get_status", sys![get_status])
            .route("start", sys![start])
            .route("stop", sys![stop])
            .route("get_output", sys![get_output])
            .route("send_command", sys![send_command]),
    );

    println!("Server is running on '0.0.0.0:8080'");

    let mut cache = TypeCache::new();

    cache.insert::<Configuration>(Arc::new(RwLock::new(Configuration::get_or_create())));
    cache.insert::<ProcessCache>(Arc::new(RwLock::new(ProcessCache::default())));

    run_with_cache("0.0.0.0:8080", router, cache);
}
