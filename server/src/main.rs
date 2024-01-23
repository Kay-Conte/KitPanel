mod authentication;
mod fs;
mod json;
mod process;
mod server_config;

use std::{
    collections::HashMap,
    process::{Command, Stdio},
    sync::{Arc, RwLock},
    time::Duration,
};

use authentication::{clean_auth, Control, Perm, User, View};
use foxhole::{
    action::RawResponse,
    framework::run_with_cache,
    resolve::{Get, Post, Query, UrlPart},
    sys,
    type_cache::{TypeCache, TypeCacheKey},
    IntoResponse, Route,
};
use fs::Config;
use json::Json;
use models::{
    GlobalStatus, InputCommandRequest, ServerOutput, ServerStatus, TokenRequest, TokenResponse,
};
use process::Process;
use server_config::ServerConfig;

use crate::authentication::Authentication;

const SESSION_LENGTH: Duration = Duration::from_secs(7200);

fn shared<T>(other: T) -> Arc<RwLock<T>> {
    Arc::new(RwLock::new(other))
}

#[derive(Default)]
pub struct ProcessManager(HashMap<String, Process>);

impl TypeCacheKey for ProcessManager {
    type Value = Arc<RwLock<ProcessManager>>;
}

fn get_all_status(
    _g: Get,
    Query(config): Query<ServerConfig>,
    Query(running): Query<ProcessManager>,
    Perm(View(scope)): Perm<View>,
) -> Json<GlobalStatus> {
    let running = &running.read().unwrap().0;
    let config = config.read().unwrap();

    let servers = config
        .servers
        .iter()
        .filter(|i| scope.contains(&i.id))
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
    Query(config): Query<ServerConfig>,
    Query(running): Query<ProcessManager>,
    Perm(Control(scope)): Perm<Control>,
) -> u16 {
    if !scope.contains(&server_id) {
        return 401;
    }

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

    let dir = config.server_directory.join(server.id.clone());

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

fn stop(
    _p: Post,
    UrlPart(server_id): UrlPart,
    Query(running): Query<ProcessManager>,
    Perm(Control(scope)): Perm<Control>,
) -> u16 {
    if !scope.contains(&server_id) {
        return 401;
    }

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
    Query(running): Query<ProcessManager>,
    Perm(View(scope)): Perm<View>,
) -> RawResponse {
    if !scope.contains(&server_id) {
        return 401u16.response();
    }

    let running = running.read().unwrap();

    let Some(server) = running.0.get(&server_id) else {
        return Json(ServerOutput { output: None }).response();
    };

    let output = server.console.inner();

    Json(ServerOutput {
        output: Some(output),
    })
    .response()
}

fn input(
    _p: Post,
    UrlPart(server_id): UrlPart,
    Json(command): Json<InputCommandRequest>,
    Query(processes): Query<ProcessManager>,
    user: User,
    Perm(Control(scope)): Perm<Control>,
) -> u16 {
    if !scope.contains(&server_id) {
        return 401;
    }

    let mut processes = processes.write().unwrap();

    let Some(process) = processes.0.get_mut(&server_id) else {
        return 200;
    };

    process.send(command.command, Some(user.user_id));

    200
}

fn auth(
    _g: Get,
    Json(request): Json<TokenRequest>,
    Query(authentication): Query<Authentication>,
) -> Json<TokenResponse> {
    let user = {
        let auth = authentication.read().unwrap();

        let Some(user) = auth.get_user(&request.username, &request.password) else {
            return Json(TokenResponse { token: None });
        };

        user.clone()
    };

    let mut auth = authentication.write().unwrap();

    let token = auth.create_session(&user.user_id);

    let res = Json(TokenResponse { token: Some(token) });

    res
}

fn version(_g: Get) -> Json<String> {
    Json(env!("CARGO_PKG_VERSION").to_string())
}

fn main() {
    let router = Route::empty().route("web", sys![]).route(
        "api",
        Route::empty()
            .route("version", sys![version])
            .route("auth", sys![auth])
            .route("status", sys![get_all_status])
            .route(
                "server",
                Route::empty()
                    .route("start", sys![start])
                    .route("stop", sys![stop])
                    .route("output", sys![get_output])
                    .route("input", sys![input]),
            ),
    );

    let mut cache = TypeCache::new();

    let auth = shared(Authentication::get().expect("Failed to create users config"));

    let auth_cloned = auth.clone();

    std::thread::spawn(|| clean_auth(auth_cloned));

    let config = ServerConfig::get().expect("Failed to construct server config");

    let address = format!("{}:{}", config.address, config.port);

    cache.insert::<ServerConfig>(shared(config));
    cache.insert::<ProcessManager>(shared(ProcessManager::default()));
    cache.insert::<Authentication>(auth);

    run_with_cache(address, router, cache);
}
