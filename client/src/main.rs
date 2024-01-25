#![windows_subsystem = "windows"]

mod cache;
mod components;
mod fs;
mod request;
mod servers;
mod settings;
mod tab_nav;
mod theme;
mod views;

use std::time::Duration;

use cache::Cache;
use components::{status_bar, Status};
use fs::Config;
use iced::{
    executor,
    font::{self, Family},
    keyboard::{self, KeyCode},
    subscription::events,
    widget::{text_input, Text},
    window::{self, resize},
    Application, Command, Event, Font, Renderer, Settings, Size, Subscription,
};

use request::Request;
use servers::Servers;
use theme::Theme;

use uuid::Uuid;
use views::{
    home::{self, MainState},
    login::{self, LoginState},
    settings::SettingsState,
};

pub const EXPAND_ARROW: &'static [u8] = include_bytes!("../assets/icons/ExpandArrow.png");
pub const EXPAND_ARROW_CLOSED: &'static [u8] =
    include_bytes!("../assets/icons/ExpandArrowClosed.png");
pub const LOGO: &'static [u8] = include_bytes!("../assets/icons/KitPanelLogo.png");
pub const POWER_BUTTON: &'static [u8] = include_bytes!("../assets/icons/PowerButton.png");
pub const LOGOUT_BUTTON: &'static [u8] = include_bytes!("../assets/icons/LogoutButton.png");
pub const SETTINGS_BUTTON: &'static [u8] = include_bytes!("../assets/icons/SettingsButton.png");
pub const BACK_ARROW: &'static [u8] = include_bytes!("../assets/icons/BackArrow.png");

type Element<'a, M> = iced::Element<'a, M, Renderer<Theme>>;

#[derive(Debug, Clone)]
pub enum Message {
    LoginPage(login::Event),
    HomePage(home::Event),
    SettingsPage(views::settings::Event),

    Event(Event),

    GotoPrevious,
    GotoPage(Page),

    Login(String, String, String),
    LoggedIn(Uuid, String, String),

    FontLoaded(Result<(), font::Error>),

    Error(String),
    ResetStatus(Status),

    None,
}

#[derive(Debug, Clone)]
pub enum Page {
    Login(LoginState),
    Main(MainState),
    Settings(SettingsState),
}

impl Default for Page {
    fn default() -> Self {
        Self::Login(LoginState::default())
    }
}

impl Page {
    fn window_size(&self) -> Option<Size<u32>> {
        match self {
            Page::Login(_) => Some(Size {
                width: 512,
                height: 768,
            }),
            Page::Main(_) => Some(Size {
                width: 768,
                height: 768,
            }),
            Page::Settings(_) => Some(Size {
                width: 768,
                height: 768,
            }),
        }
    }
}

struct App {
    page: Page,
    previous_page: Option<Page>,
    status_bar: Status,
    login_cache: Cache,
}

impl Application for App {
    type Executor = executor::Default;

    type Message = Message;

    type Theme = Theme;

    type Flags = Cache;

    fn new(flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                page: Page::Login(LoginState {
                    address: flags.last_address.clone(),
                    username: flags.last_username.clone(),
                    ..Default::default()
                }),
                previous_page: None,
                status_bar: Status::None,
                login_cache: flags,
            },
            Command::batch(vec![font::load(
                include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf").as_slice(),
            )
            .map(|i| Message::FontLoaded(i))]),
        )
    }

    fn title(&self) -> String {
        "KitPanel".to_string()
    }

    fn theme(&self) -> Self::Theme {
        Theme::default()
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        let mut commands = vec![];

        match message {
            Message::HomePage(e) => {
                let Page::Main(state) = &mut self.page else {
                    return Command::batch(commands);
                };

                let (msg, cmd) = state.update(e);

                commands.push(cmd.map(Message::HomePage));

                if let Some(m) = msg {
                    let command = self.update(m);

                    commands.push(command);
                }
            }

            Message::LoginPage(e) => {
                let Page::Login(state) = &mut self.page else {
                    return Command::batch(commands);
                };

                let (msg, cmd) = state.update(e);

                commands.push(cmd.map(Message::LoginPage));

                if let Some(m) = msg {
                    let command = self.update(m);

                    commands.push(command);
                }
            }

            Message::SettingsPage(e) => {
                let Page::Settings(state) = &mut self.page else {
                    return Command::batch(commands);
                };

                let msg = state.update(e);

                if let Some(m) = msg {
                    let command = self.update(m);

                    commands.push(command);
                }
            }

            Message::GotoPage(mut page) => {
                if let Some(size) = page.window_size() {
                    commands.push(resize(size));
                }

                std::mem::swap(&mut self.page, &mut page);

                self.previous_page = Some(page);
            }

            Message::GotoPrevious => {
                if let Some(page) = &mut self.previous_page {
                    std::mem::swap(&mut self.page, page);
                }

                if let Some(size) = self.page.window_size() {
                    commands.push(resize(size));
                }
            }

            Message::Login(address, username, password) => {
                self.login_cache = Cache::new(address.clone(), username.clone());

                let _ = self.login_cache.save();

                let request = Request::new(address.clone());
                let request_ = request.clone();

                let (username, password) = (username.clone(), password.clone());

                commands.push(Command::perform(
                    async move { request_.get_version().await },
                    |v| match v {
                        Some(version) if version == env!("CARGO_PKG_VERSION") => Message::None,
                        Some(_) => Message::Error(
                            "Version mismatch, some functionality may be missing".to_string(),
                        ),
                        _ => Message::None,
                    },
                ));

                let username_ = username.clone();

                commands.push(Command::perform(
                    async move { request.get_token(username, password).await },
                    move |i| match i {
                        Some(status) => Message::LoggedIn(status, address, username_),
                        None => Message::Error("Failed to login".to_string()),
                    },
                ));
            }

            Message::LoggedIn(token, address, username) => {
                let request = Request::new(address.clone());
                let username = username.clone();

                let request_ = request.clone();

                commands.push(Command::perform(
                    async move {
                        let status = request_.get_status(token.clone()).await;

                        status.map(Servers::from)
                    },
                    move |i| match i {
                        Some(i) => Message::GotoPage(Page::Main(MainState {
                            request,
                            username,
                            token,
                            servers: i,
                        })),
                        None => Message::Error("Failed to load status".to_string()),
                    },
                ));
            }

            Message::Event(Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: KeyCode::Tab,
                modifiers,
            })) => {
                match &mut self.page {
                    Page::Login(state) => {
                        if modifiers.shift() {
                            commands.push(text_input::focus(state.tab_nav.back()));
                        } else {
                            commands.push(text_input::focus(state.tab_nav.next()));
                        }
                    }
                    _ => {}
                };
            }

            Message::Error(msg) => {
                let status = Status::Error(msg);

                self.status_bar = status.clone();

                commands.push(Command::perform(
                    async move { tokio::time::sleep(Duration::from_secs(5)).await },
                    |_| Message::ResetStatus(status),
                ))
            }

            Message::ResetStatus(status) => {
                if self.status_bar == status {
                    self.status_bar = Status::None;
                }
            }

            _ => {}
        }

        Command::batch(commands)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let mut subscriptions = vec![];

        subscriptions.push(events().map(Message::Event));

        match &self.page {
            Page::Main(state) => subscriptions.push(state.subscription().map(Message::HomePage)),
            _ => {}
        }

        Subscription::batch(subscriptions)
    }

    fn scale_factor(&self) -> f64 {
        0.8
    }

    fn view(&self) -> Element<'_, Message> {
        let page = match &self.page {
            Page::Login(s) => s.view().map(Message::LoginPage),
            Page::Main(s) => s.view().map(Message::HomePage),
            Page::Settings(s) => s.view().map(Message::SettingsPage),
            _ => Text::new("This page is not currently used!").into(),
        };

        iced::widget::column!(page, status_bar(&self.status_bar)).into()
    }
}

fn main() {
    let cache = Cache::get().expect("Failed to get or create cache");

    let _ = App::run(Settings {
        default_font: Font {
            family: Family::Name("JetBrains Mono"),
            ..Default::default()
        },
        window: window::Settings {
            size: (512, 768),
            min_size: Some((512, 768)),
            ..Default::default()
        },
        flags: cache,
        ..Default::default()
    });
}
