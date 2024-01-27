use iced::widget::{self, image::Handle, *};

use crate::{settings::Settings, theme, Element, Message, BACK_ARROW, components::icon_button};

#[derive(Debug, Clone)]
pub enum Event {
    GotoPrevious,

    Set(bool),
}

#[derive(Default, Debug, Clone)]
pub struct SettingsState {}

impl SettingsState {
    pub fn update(&mut self, evt: Event) -> Option<Message> {
        let mut msg = None;

        match evt {
            Event::GotoPrevious => msg = Some(Message::GotoPrevious),
            _ => {}
        }

        msg
    }

    pub fn view<'a>(&self, settings: &Settings) -> Element<'a, Event> {
        let back_button =
            icon_button(Image::new(Handle::from_memory(BACK_ARROW))).on_press(Event::GotoPrevious);

        let nav_row = row!(back_button).padding(20);

        widget::column!(nav_row).into()
    }
}
