use std::{
    collections::HashMap,
    marker::PhantomData,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use foxhole::{
    resolve::{Resolve, ResolveGuard},
    type_cache::TypeCacheKey,
    IntoResponse, Response,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{fs::Config, SESSION_LENGTH};

pub type UserId = String;
pub type Password = String;
pub type Token = Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum Scope {
    All,
    Some(Vec<String>),
}

impl Default for Scope {
    fn default() -> Self {
        Self::Some(vec![])
    }
}

impl Scope {
    pub fn contains(&self, other: &String) -> bool {
        match self {
            Scope::All => true,
            Scope::Some(i) => i.contains(other)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct Permissions {
    admin: bool,

    edit: Scope,
    view: Scope,
    control: Scope,
}

impl Permissions {
    fn admin() -> Self {
        Self::default()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub user_id: UserId,
    pub permissions: Permissions,
    pub password: String,
}

pub struct Session {
    started: Instant,
    user_id: UserId,
}

impl Session {
    fn new(user_id: UserId) -> Self {
        Self {
            started: Instant::now(),
            user_id,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Authentication {
    pub users: HashMap<UserId, User>,

    #[serde(skip_serializing, skip_deserializing)]
    pub sessions: HashMap<Token, Session>,
}

impl Default for Authentication {
    fn default() -> Self {
        Self::template()
    }
}

impl Config for Authentication {
    fn rel_path(rel: std::path::PathBuf) -> std::path::PathBuf {
        rel.join("accounts.json")
    }

    fn bytes(&self) -> Vec<u8> {
        serde_json::to_vec_pretty(self).unwrap()
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        serde_json::from_slice(bytes).ok()
    }
}

impl TypeCacheKey for Authentication {
    type Value = Arc<RwLock<Authentication>>;
}

impl Authentication {
    pub fn template() -> Self {
        let mut authentication = Authentication {
            users: HashMap::new(),
            sessions: HashMap::new(),
        };

        authentication.users.insert(
            "admin".to_string(),
            User {
                user_id: "admin".to_string(),
                permissions: Permissions::admin(),
                password: "password".to_string(),
            },
        );

        authentication
    }

    pub fn get_user(&self, user_id: &UserId, password: &Password) -> Option<&User> {
        self.users.get(user_id).filter(|i| i.password == *password)
    }

    pub fn create_session(&mut self, user_id: &str) -> Token {
        let token = Uuid::new_v4();

        let session = Session::new(user_id.to_owned());

        self.sessions.insert(token.clone(), session);

        token
    }

    pub fn clean(&mut self, timeout: Duration) {
        let now = Instant::now();

        self.sessions = std::mem::take(&mut self.sessions)
            .into_iter()
            .filter(|session| (now - session.1.started) <= timeout)
            .collect();
    }
}

pub fn clean_auth(auth: Arc<RwLock<Authentication>>) {
    loop {
        std::thread::sleep(Duration::from_secs(120));

        auth.write().unwrap().clean(SESSION_LENGTH);
    }
}

trait Permission {
    fn get_permission(permissions: &Permissions) -> Self;
}

pub struct Perm<T>(pub T);

impl<'a, T> Resolve<'a> for Perm<T>
where
    T: 'a + Permission,
{
    type Output = Perm<T>;

    fn resolve(
        ctx: &'a foxhole::RequestState,
        _path_iter: &mut foxhole::PathIter,
    ) -> foxhole::resolve::ResolveGuard<Self::Output> {
        let Some(Ok(token)) = ctx.request.headers().get("authorization").map(|i| i.to_str()) else {
            return ResolveGuard::Respond(401u16.response());
        };

        let Ok(token) = serde_json::from_str(token) else {
            return ResolveGuard::Respond(401u16.response());
        };

        let cache = ctx.global_cache.read().unwrap();

        let auth = cache.get::<Authentication>().unwrap().read().unwrap();

        let Some(session) = auth.sessions.get(&token) else {
            return ResolveGuard::Respond(401u16.response());
        };

        let Some(user) = auth.users.get(&session.user_id) else {
            return ResolveGuard::Respond(401u16.response());
        };

        ResolveGuard::Value(Perm(T::get_permission(&user.permissions)))
    }
}

pub struct Edit(pub Scope);

impl Permission for Edit {
    fn get_permission(permissions: &Permissions) -> Self {
        Self(permissions.edit.clone())
    }
}

pub struct View(pub Scope);

impl Permission for View {
    fn get_permission(permissions: &Permissions) -> Self {
        Self(permissions.view.clone())
    }
}

pub struct Control(pub Scope);

impl Permission for Control {
    fn get_permission(permissions: &Permissions) -> Self {
        Self(permissions.control.clone())
    }
}
