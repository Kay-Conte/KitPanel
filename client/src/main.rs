#![windows_subsystem = "windows"]

mod cache;
mod components;
mod fs;
mod request;
mod theme;

use std::time::Duration;

use cache::Cache;
use components::{navbar, status_bar, Card, Status};
use fs::Config;
use iced::{
    executor,
    font::{self, Family},
    keyboard::{self, KeyCode},
    subscription::{self, events},
    widget::{
        button, column, image::Handle, row, scrollable, text_input, Column, Container, Image,
        Space, Text,
    },
    window::{self, resize},
    Alignment, Application, Command, Element, Event, Font, Length, Renderer, Settings, Size,
    Subscription,
};

use indexmap::IndexMap;
use models::{GlobalStatus, ServerOutput, ServerStatus};
use request::Request;
use theme::Theme;

use uuid::Uuid;

pub const EXPAND_ARROW: &'static [u8] = include_bytes!("../assets/icons/ExpandArrow.png");
pub const EXPAND_ARROW_CLOSED: &'static [u8] =
    include_bytes!("../assets/icons/ExpandArrowClosed.png");
pub const LOGO: &'static [u8] = include_bytes!("../assets/icons/KitPanelLogo.png");
pub const POWER_BUTTON: &'static [u8] = include_bytes!("../assets/icons/PowerButton.png");
pub const LOGOUT_BUTTON: &'static [u8] = include_bytes!("../assets/icons/LogoutButton.png");

#[derive(Debug, Clone)]
struct Server {
    id: String,
    running: bool,
    output: Vec<String>,
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
    fn update(&mut self, server_status: ServerStatus) {
        self.running = server_status.running;
    }
}

#[derive(Debug, Clone)]
pub struct Servers {
    inner: IndexMap<String, Server>,
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

    fn update(&mut self, global_status: GlobalStatus) {
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

#[derive(Debug, Clone, Copy)]
pub enum LoginField {
    Address,
    Username,
    Password,
}

#[derive(Debug, Clone)]
struct TabNav {
    ordered: Vec<text_input::Id>,
    current: usize,
}

impl TabNav {
    fn new(ordered: Vec<text_input::Id>) -> Self {
        Self {
            ordered,
            current: 0,
        }
    }

    fn set(&mut self, id: text_input::Id) {
        if let Some(idx) = self.ordered.iter().position(|i| *i == id) {
            self.current = idx;
        }
    }

    fn next(&mut self) -> text_input::Id {
        self.current = (self.current + 1) % self.ordered.len();

        self.ordered[self.current].clone()
    }

    fn back(&mut self) -> text_input::Id {
        if self.current == 0 {
            self.current = self.ordered.len() - 1;
        } else {
            self.current = self.current - 1
        }

        self.ordered[self.current].clone()
    }
}

#[derive(Debug, Clone)]
pub struct MainState {
    request: Request,
    username: String,

    token: Uuid,
    servers: Servers,
}

#[derive(Debug, Clone)]
pub struct LoginState {
    address: String,

    username: String,
    password: String,

    tab_nav: TabNav,
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

#[derive(Debug, Clone)]
pub enum Message {
    Event(Event),

    LoggedIn(Uuid),

    GotoPrevious,

    GotoPage(Page),

    FontLoaded(Result<(), font::Error>),

    UpdateLoginInput(LoginField, String),

    RefreshStatus,
    OutputRefreshed(String, Vec<String>),
    StatusRefreshed(GlobalStatus),

    ToggleServer(String),
    SendCommand(String, String),

    SetFocus(text_input::Id),

    Error(String),
    ResetStatus(Status),

    None,
    SubmitLogin,
    Logout,
}

#[derive(Debug, Clone)]
pub enum Page {
    Login(LoginState),
    Main(MainState),
    Settings,
}

impl Default for Page {
    fn default() -> Self {
        Self::Login(LoginState::default())
    }
}

impl Page {
    fn window_size(&self) -> Option<Size<u32>> {
        match self {
            Page::Main(_) => Some(Size {
                width: 768,
                height: 768,
            }),
            Page::Login(_) => Some(Size {
                width: 512,
                height: 768,
            }),
            _ => None,
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
            Message::GotoPage(page) => {
                match page.window_size() {
                    Some(size) => commands.push(resize(size)),
                    None => {}
                }

                self.page = page;
            }
            Message::SubmitLogin => {
                let Page::Login(state) = &mut self.page else {
                    return Command::batch(commands);
                };

                self.login_cache = Cache::new(state.address.clone(), state.username.clone());

                let _ = self.login_cache.save();

                let request = Request::new(state.address.clone());
                let request_ = request.clone();

                let (username, password) = (state.username.clone(), state.password.clone());

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

                commands.push(Command::perform(
                    async move { request.get_token(username, password).await },
                    |i| match i {
                        Some(status) => Message::LoggedIn(status),
                        None => Message::Error("Failed to login".to_string()),
                    },
                ));
            }

            Message::LoggedIn(token) => {
                let Page::Login(state) = &mut self.page else {
                    return Command::batch(commands);
                };

                let request = Request::new(state.address.clone());
                let username = state.username.clone();

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
            Message::UpdateLoginInput(field, s) => {
                let Page::Login(state) = &mut self.page else {
                    return Command::batch(commands);
                };

                match field {
                    LoginField::Address => state.address = s,
                    LoginField::Username => state.username = s,
                    LoginField::Password => state.password = s,
                }
            }
            Message::GotoPrevious => {
                let Some(mut page) = self.previous_page.take() else {
                    return Command::batch(commands);
                };

                std::mem::swap(&mut self.page, &mut page);

                self.previous_page = Some(page);
            }
            Message::Logout => {
                self.page = Page::Login(LoginState {
                    username: self.login_cache.last_username.clone(),
                    address: self.login_cache.last_address.clone(),
                    ..Default::default()
                });

                commands.push(resize(self.page.window_size().unwrap()));
            }
            Message::RefreshStatus => {
                let Page::Main(state) = &mut self.page else {
                    return Command::batch(commands);
                };

                let request = state.request.clone();

                let token = state.token.clone();

                commands.push(Command::perform(
                    async move { request.get_status(token).await },
                    |i| match i {
                        Some(status) => Message::StatusRefreshed(status),
                        None => Message::Error("Failed to load status".to_string()),
                    },
                ))
            }
            Message::StatusRefreshed(status) => match &mut self.page {
                Page::Main(state) => state.servers.update(status),
                _ => {}
            },
            Message::OutputRefreshed(id, new) => {
                let Page::Main(state) = &mut self.page else {
                    return Command::batch(commands);
                };

                let Some(server) = state.servers.inner.get_mut(&id) else {
                    return Command::batch(commands);
                };

                server.output = new;
            }
            Message::ToggleServer(server_id) => {
                let Page::Main(state) = &mut self.page else {
                    return Command::batch(commands);
                };

                let request = state.request.clone();

                let Some(server) = state.servers.inner.get(&server_id) else {
                    return Command::batch(commands);
                };

                let token = state.token.clone();

                match server.running {
                    true => commands.push(Command::perform(
                        async move { request.stop_server(server_id, token).await },
                        |_i| Message::RefreshStatus,
                    )),
                    false => commands.push(Command::perform(
                        async move { request.start_server(server_id, token).await },
                        |_i| Message::RefreshStatus,
                    )),
                };
            }
            Message::SendCommand(server_id, command) => {
                let Page::Main(state) = &mut self.page else {
                    return Command::batch(commands);
                };

                let request = state.request.clone();

                let token = state.token.clone();

                commands.push(Command::perform(
                    async move { request.send_command(server_id, command, token).await },
                    |_i| Message::None,
                ))
            }
            Message::SetFocus(id) => match &mut self.page {
                Page::Login(state) => state.tab_nav.set(id),
                _ => {}
            },
            Message::FontLoaded(_r) => {}
            Message::Error(msg) => {
                let status = Status::Error(msg);

                self.status_bar = status.clone();

                commands.push(Command::perform(
                    async move { tokio::time::sleep(Duration::from_secs(5)).await },
                    |_| Message::ResetStatus(status),
                ))
            }
            Message::ResetStatus(status) if self.status_bar == status => {
                self.status_bar = Status::None
            }
            _ => {}
        }

        Command::batch(commands)
    }

    fn scale_factor(&self) -> f64 {
        0.8
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let mut subscriptions = vec![];

        subscriptions.push(events().map(Message::Event));

        let Page::Main(ref state) = self.page else {
            return Subscription::batch(subscriptions);
        };

        subscriptions.push(subscription::unfold(
            "refresh_status".to_string(),
            (state.request.clone(), state.token.clone()),
            refresh_status,
        ));

        for server in state.servers.inner.values() {
            subscriptions.push(subscription::unfold(
                server.id.clone(),
                (
                    server.id.clone(),
                    state.request.clone(),
                    state.token.clone(),
                ),
                refresh_output,
            ))
        }

        Subscription::batch(subscriptions)
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        match &self.page {
            Page::Login(state) => self.login_page(state),
            Page::Main(state) => self.main_page(state),
            Page::Settings => self.settings_page(),
        }
    }
}

impl App {
    fn settings_page<'a>(&self) -> Element<'a, Message, Renderer<Theme>> {
        let navbar = row!();

        column!(navbar).into()
    }

    fn main_page<'a>(&self, state: &MainState) -> Element<'a, Message, Renderer<Theme>> {
        let username = Text::new(state.username.clone()).size(30);

        let logout_icon = Image::new(Handle::from_memory(LOGOUT_BUTTON));

        let logout_button = button(logout_icon)
            .on_press(Message::Logout)
            .style(theme::Button::Transparent);

        let nav = navbar(row!(username, logout_button).spacing(24).into());

        let mut col = Column::new().spacing(2);

        for server in state.servers.inner.values() {
            col = col.push(Card {
                server_id: server.id.clone(),
                status: server.running,
                console: server.output.clone(),
            });
        }

        column!(
            nav,
            scrollable(col).height(Length::Fill),
            status_bar(self.status_bar.clone()),
        )
        .into()
    }

    fn login_page<'a>(&self, state: &LoginState) -> Element<'a, Message, Renderer<Theme>> {
        let nav = navbar(Text::new("").into());

        let handle = Handle::from_memory(LOGO);

        let logo = Image::new(handle).width(500.0);

        let input = |placeholder, value, field| {
            text_input(placeholder, value)
                .on_input(move |s| Message::UpdateLoginInput(field, s))
                .on_submit(Message::SubmitLogin)
                .padding([10, 25])
                .size(24.0)
        };

        let address_input = input("Address", &state.address, LoginField::Address)
            .width(Length::Fill)
            .id(text_input::Id::new("address"));

        let username_input = input("Username", &state.username, LoginField::Username)
            .id(text_input::Id::new("username"));
        let password_input = input("Password", &state.password, LoginField::Password)
            .id(text_input::Id::new("password"));

        let handle = Handle::from_memory(EXPAND_ARROW_CLOSED);
        let img = Container::new(Image::new(handle).height(32.0).width(32.0))
            .height(Length::Fill)
            .width(Length::Fill)
            .center_x()
            .center_y();

        let login_button = button(img)
            .on_press(Message::SubmitLogin)
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
            Space::new(Length::Fill, Length::Fill),
            status_bar(self.status_bar.clone()),
        )
        .align_items(Alignment::Center)
        .into()
    }
}

async fn refresh_status(state: (Request, Uuid)) -> (Message, (Request, Uuid)) {
    tokio::time::sleep(Duration::from_secs(1)).await;

    let (request, token) = &state;

    let Some(global_status) = request.get_status(token.clone()).await else {
        return (
            Message::Error("Failed to retrieve status from remote".to_string()),
            state,
        );
    };

    (Message::StatusRefreshed(global_status), state)
}

async fn refresh_output(state: (String, Request, Uuid)) -> (Message, (String, Request, Uuid)) {
    tokio::time::sleep(Duration::from_secs(1)).await;

    let (server_id, request, token) = &state;

    let Some(ServerOutput {
        output: Some(output),
    }) = request.get_output(server_id.clone(), token.clone()).await
    else {
        return (Message::None, state);
    };

    (Message::OutputRefreshed(server_id.clone(), output), state)
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
