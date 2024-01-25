use iced::widget::*;

use crate::{Element, Message};

#[derive(Debug, Clone)]
pub enum Event {
    Set(bool),
}

#[derive(Debug, Default, Clone)]
pub struct SettingsState {
    set: bool,
}

impl SettingsState {
    pub fn update(&mut self, evt: Event) -> Option<Message> {
        let msg = None;

        match evt {
            Event::Set(v) => self.set = v,
        }

        msg
    }

    pub fn view<'a>(&self) -> Element<'a, Event> {
        Toggler::new(None, self.set, Event::Set).size(48).into()
    }
}
