mod home;

use crate::{Element, Message};

pub trait Page {
    type Event: Clone;

    fn update(&mut self, event: Self::Event) -> Option<Message>;

    fn view<'a>(&self) -> Element<'a>;
}

