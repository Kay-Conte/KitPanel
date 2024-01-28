use iced::{
    widget::{self, image::Handle, *},
    Length, Pixels,
};

use crate::{
    components::{icon_button, settings_card, tab_bar, Tab},
    settings::Settings,
    theme, Element, Message, SettingsField, BACK_ARROW,
};

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Page {
    #[default]
    Client,
    Server,
}

impl Page {
    fn is_client(&self) -> bool {
        *self == Page::Client
    }

    fn is_server(&self) -> bool {
        *self == Page::Server
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    GotoPrevious,

    GotoPage(Page),

    SetCache(bool),

    Set(bool),
    SetDarkMode(bool),
}

#[derive(Default, Debug, Clone)]
pub struct SettingsState {
    page: Page,
}

impl SettingsState {
    pub fn update(&mut self, evt: Event) -> Option<Message> {
        let mut msg = None;

        match evt {
            Event::GotoPrevious => msg = Some(Message::GotoPrevious),

            Event::GotoPage(s) => self.page = s,

            Event::SetCache(v) => msg = Some(Message::UpdateSettings(SettingsField::Cache(v))),
            Event::SetDarkMode(v) => {
                msg = Some(Message::UpdateSettings(SettingsField::DarkMode(v)))
            }

            _ => {}
        }

        msg
    }

    pub fn view<'a>(&self, settings: &Settings) -> Element<'a, Event> {
        let back_button =
            icon_button(Image::new(Handle::from_memory(BACK_ARROW))).on_press(Event::GotoPrevious);

        let tabs = tab_bar(vec![
            Tab::new("Client")
                .selected(self.page.is_client())
                .on_select(Event::GotoPage(Page::Client)),
            Tab::new("Server")
                .selected(self.page.is_server())
                .on_select(Event::GotoPage(Page::Server)),
        ]);

        let width = iced::advanced::Widget::width(&back_button);

        let nav_row = row!(
            back_button,
            Space::new(Length::Fill, 0.0),
            tabs,
            Space::new(Length::Fill, 0.0),
            Space::new(width, 0.0)
        )
        .padding(20);

        let cache = settings_card(
            "Cache Data",
            Some(
                "Whether or not you would like to cache login data for faster logins.".to_string(),
            ),
            Toggler::new(None, settings.enable_cache, Event::SetCache)
                .width(64)
                .size(32),
        );

        let dark_mode = settings_card(
            "Dark Mode",
            None,
            Toggler::new(None, settings.dark_mode, Event::SetDarkMode)
                .width(64)
                .size(32),
        );

        let settings = widget::column!(dark_mode, cache).spacing(25).padding([0, 120]);

        widget::column!(nav_row, settings).into()
    }
}
