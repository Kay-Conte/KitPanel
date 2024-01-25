use indexmap::IndexMap;
use models::{ServerStatus, GlobalStatus};

#[derive(Debug, Clone)]
pub struct Server {
    pub id: String,
    pub running: bool,
    pub output: Vec<String>,
}

impl From<ServerStatus> for Server {
    fn from(value: ServerStatus) -> Self {
        Server {
            id: value.id,
            running: value.running,
            output: Vec::new(),
        }
    }
}

impl Server {
    pub fn update(&mut self, server_status: ServerStatus) {
        self.running = server_status.running;
    }
}

#[derive(Debug, Clone)]
pub struct Servers {
    pub inner: IndexMap<String, Server>,
}

impl From<GlobalStatus> for Servers {
    fn from(value: GlobalStatus) -> Servers {
        let mut servers = Servers::new();

        for server in value.servers {
            let server = Server::from(server);

            servers.inner.insert(server.id.clone(), server);
        }

        servers
    }
}

impl Servers {
    fn new() -> Self {
        Self {
            inner: IndexMap::new(),
        }
    }

    pub fn update(&mut self, global_status: GlobalStatus) {
        self.inner = self
            .inner
            .clone()
            .into_iter()
            .filter(|i| {
                global_status
                    .servers
                    .iter()
                    .find(|p| p.id == i.1.id)
                    .is_some()
            })
            .collect();

        for server_status in global_status.servers {
            let Some(server) = self.inner.get_mut(&server_status.id) else {
                self.inner
                    .insert(server_status.id.clone(), Server::from(server_status));
                continue;
            };

            server.update(server_status);
        }
    }
}
