use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use foxhole::type_cache::TypeCacheKey;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type UserId = String;
pub type Password = String;
pub type Token = Uuid;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Edit,
    View,
}

impl Permission {
    fn admin() -> Vec<Self> {
        vec![Self::Edit, Self::View]
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub user_id: UserId,
    pub permissions: Vec<Permission>,
    pub password: String,
}

impl User {
    pub fn is_perm(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }
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

#[derive(Serialize, Deserialize, Default)]
pub struct Authentication {
    pub users: HashMap<UserId, User>,

    #[serde(skip_serializing, skip_deserializing)]
    pub sessions: HashMap<Token, Session>,
}

impl TypeCacheKey for Authentication {
    type Value = Arc<RwLock<Authentication>>;
}

impl Authentication {
    pub fn template() -> Self {
        let mut authentication = Authentication::default();

        authentication.users.insert(
            "admin".to_string(),
            User {
                user_id: "admin".to_string(),
                permissions: Permission::admin(),
                password: "password".to_string(),
            },
        );

        authentication
    }

    pub fn get_user(&self, user_id: &UserId, password: &Password) -> Option<&User> {
        self.users.get(user_id).filter(|i| i.password == *password)
    }

    pub fn get_session(&self, token: &Token) -> Option<&Session> {
        self.sessions.get(token)
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
            .filter(|session| (now - session.1.started) >= timeout)
            .collect();
    }
}
