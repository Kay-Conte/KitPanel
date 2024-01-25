use iced::{
    widget::{button, column, image::Handle, row, text_input, Container, Image, Space},
    Alignment, Length, Command,
};

use crate::{
    components::navbar, tab_nav::TabNav, theme, Element, Message, Page, EXPAND_ARROW_CLOSED, LOGO,
    SETTINGS_BUTTON,
};

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginField {
    Address,
    Username,
    Password,
}

#[derive(Debug, Clone)]
pub enum Event {
    Super(Box<Message>),
    UpdateLoginInput(LoginField, String),
    SubmitLogin,

    Nav(bool),
}

#[derive(Debug, Clone)]
pub struct LoginState {
    pub address: String,

    pub username: String,
    pub password: String,

    pub tab_nav: TabNav,
}

impl Default for LoginState {
    fn default() -> Self {
        Self {
            address: String::new(),

            username: String::new(),
            password: String::new(),

            tab_nav: TabNav::new(vec![
                text_input::Id::new("address"),
                text_input::Id::new("username"),
                text_input::Id::new("password"),
            ]),
        }
    }
}

impl LoginState {
    pub fn update(&mut self, evt: Event) -> (Option<Message>, Command<Event>) {
        let mut msg = None;
        let mut commands = vec![];

        match evt {
            Event::Super(m) => msg = Some(*m),
            Event::UpdateLoginInput(field, value) => match field {
                LoginField::Address => self.address = value,
                LoginField::Username => self.username = value,
                LoginField::Password => self.password = value,
            },
            Event::SubmitLogin => {
                msg = Some(Message::Login(
                    self.address.clone(),
                    self.username.clone(),
                    self.password.clone(),
                ));

                self.password.clear();
            }

            Event::Nav(forwards) => {
                let cmd = if forwards {
                    text_input::focus(self.tab_nav.next())
                } else {
                    text_input::focus(self.tab_nav.back())
                };

                commands.push(cmd);
            }
        }

        (msg, Command::batch(commands))
    }

    pub fn view<'a>(&self) -> Element<'a, Event> {
        let settings_icon = Image::new(Handle::from_memory(SETTINGS_BUTTON));

        let settings_button = button(settings_icon)
            .style(theme::Button::Transparent)
            .on_press(Event::Super(Box::new(Message::GotoPage(Page::Settings))));

        let nav = navbar(settings_button.into());

        let handle = Handle::from_memory(LOGO);

        let logo = Image::new(handle).width(500.0);

        let input = |placeholder, value, field| {
            text_input(placeholder, value)
                .on_input(move |s| Event::UpdateLoginInput(field, s))
                .on_submit(Event::SubmitLogin)
                .padding([10, 25])
                .size(24.0)
        };

        let address_input = input("Address", &self.address, LoginField::Address)
            .width(Length::Fill)
            .id(text_input::Id::new("address"));

        let username_input = input("Username", &self.username, LoginField::Username)
            .id(text_input::Id::new("username"));
        let password_input = input("Password", &self.password, LoginField::Password)
            .id(text_input::Id::new("password"));

        let handle = Handle::from_memory(EXPAND_ARROW_CLOSED);
        let img = Container::new(Image::new(handle).height(32.0).width(32.0))
            .height(Length::Fill)
            .width(Length::Fill)
            .center_x()
            .center_y();

        let login_button = button(img)
            .on_press(Event::SubmitLogin)
            .height(Length::Fill)
            .width(100.0);

        let user_col = column!(username_input, password_input).width(Length::Fill);

        let user_row = row!(user_col, login_button).height(100.0);

        column!(
            nav,
            logo,
            Space::new(Length::Fill, Length::Fill),
            address_input,
            Space::new(Length::Fill, 50.0),
            user_row,
            Space::new(Length::Fill, 50.0)
        )
        .height(Length::Fill)
        .align_items(Alignment::Center)
        .into()
    }
}
