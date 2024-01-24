use uuid::Uuid;

use crate::{request::Request, Servers};

use super::Page;

pub struct Home {
    request: Request,
    username: String,
    
    token: Uuid,
    servers: Servers,
}

#[derive(Debug, Clone)]
pub enum HomeEvent {

}

impl Page for Home {
    type Event = HomeEvent;

    fn update(&mut self, event: Self::Event) -> Option<crate::Message> {
        todo!()
    }

    fn view<'a>(&self) -> crate::Element<'a> {
        todo!()
    }
}
