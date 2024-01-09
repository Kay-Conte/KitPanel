#![windows_subsystem = "windows"]

mod components;
mod request;
mod theme;

use std::time::Duration;

use components::{navbar, status_bar, Card, Status};
use iced::{
    executor,
    font::{self, Family},
    subscription,
    widget::{
        button, column, image::Handle, row, scrollable, text_input, Column, Container, Image,
        Space, Text,
    },
    window::{self, resize},
    Alignment, Application, Command, Element, Font, Length, Renderer, Settings, Size, Subscription,
};

use indexmap::IndexMap;
use models::{GlobalStatus, ServerOutput, ServerStatus};
use request::Request;
use theme::Theme;

pub const EXPAND_ARROW: &'static [u8] = include_bytes!("../assets/icons/ExpandArrow.png");
pub const EXPAND_ARROW_CLOSED: &'static [u8] =
    include_bytes!("../assets/icons/ExpandArrowClosed.png");
pub const LOGO: &'static [u8] = include_bytes!("../assets/icons/kit_panel_logo_no_bg.png");

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

    // TODO Handle removing entries that no longer exist in `GlobalStatus`
    fn update(mut self, global_status: GlobalStatus) -> Self {
        for server_status in global_status.servers {
            let Some(server) = self.inner.get_mut(&server_status.id) else {
                self.inner
                    .insert(server_status.id.clone(), Server::from(server_status));
                continue;
            };

            server.update(server_status);
        }

        self
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LoginField {
    Address,
    Username,
    Password,
}

#[derive(Debug)]
struct MainState {
    request: Request,
    username: String,
}

#[derive(Default, Debug)]
struct LoginState {
    address: String,

    username: String,
    password: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    FontLoaded(Result<(), font::Error>),

    UpdateLoginInput(LoginField, String),

    RefreshStatus,
    OutputRefreshed(String, Vec<String>),
    StatusRefreshed(GlobalStatus),

    ToggleServer(String),
    SendCommand(String, String),

    Error(String),
    None,
    SubmitLogin,
}

#[derive(Debug)]
enum Page {
    Login(LoginState),
    Main(MainState),
}

impl Default for Page {
    fn default() -> Self {
        Self::Login(LoginState::default())
    }
}

impl Page {
    fn login_state(&mut self) -> &mut LoginState {
        match self {
            Self::Login(state) => state,
            _ => panic!("Attempted to retrieve invalid state"),
        }
    }

    fn main_state(&mut self) -> &mut MainState {
        match self {
            Self::Main(state) => state,
            _ => panic!("Attempted to retrieve invalid state"),
        }
    }
}

struct App {
    page: Page,
    status: Status,
    servers: Option<Servers>,
}

impl Application for App {
    type Executor = executor::Default;

    type Message = Message;

    type Theme = Theme;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self {
                page: Page::default(),
                status: Status::None,
                servers: None,
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
            Message::UpdateLoginInput(field, s) => {
                let state = self.page.login_state();

                match field {
                    LoginField::Address => state.address = s,
                    LoginField::Username => state.username = s,
                    LoginField::Password => state.password = s,
                }
            }
            Message::SubmitLogin => {
                let state = self.page.login_state();

                let main_state = MainState {
                    request: Request::new(state.address.clone()),
                    username: state.username.clone(),
                };

                let request = main_state.request.clone();

                commands.push(Command::perform(
                    async move { request.get_status().await },
                    |i| match i {
                        Some(status) => Message::StatusRefreshed(status),
                        None => Message::Error("Failed to load status".to_string()),
                    },
                ));

                commands.push(resize(Size {
                    width: 768,
                    height: 768,
                }));

                self.page = Page::Main(main_state);
            }

            Message::RefreshStatus => {
                let state = self.page.main_state();

                let request = state.request.clone();

                commands.push(Command::perform(
                    async move { request.get_status().await },
                    |i| match i {
                        Some(status) => Message::StatusRefreshed(status),
                        None => Message::Error("Failed to load status".to_string()),
                    },
                ))
            }
            Message::StatusRefreshed(status) => {
                self.servers = Some(match self.servers.take() {
                    Some(i) => i.update(status),
                    None => Servers::from(status),
                });
            }

            Message::OutputRefreshed(id, new) => {
                let Some(servers) = &mut self.servers else {
                    return Command::none();
                };

                let Some(server) = servers.inner.get_mut(&id) else {
                    return Command::none();
                };

                server.output = new;
            }

            Message::ToggleServer(server_id) => {
                let state = self.page.main_state();

                let request = state.request.clone();

                let Some(servers) = &self.servers else {
                    return Command::none();
                };

                let Some(server) = servers.inner.get(&server_id) else {
                    return Command::none();
                };

                match server.running {
                    true => commands.push(Command::perform(
                        async move { request.stop_server(server_id).await },
                        |_i| Message::RefreshStatus,
                    )),
                    false => commands.push(Command::perform(
                        async move { request.start_server(server_id).await },
                        |_i| Message::RefreshStatus,
                    )),
                };
            }
            Message::SendCommand(server_id, command) => {
                let state = self.page.main_state();

                let request = state.request.clone();

                commands.push(Command::perform(
                    async move { request.send_command(server_id, command).await },
                    |_i| Message::None,
                ))
            }

            Message::FontLoaded(_r) => {}
            Message::Error(msg) => self.status = Status::Error(msg),
            Message::None => {}
        }

        Command::batch(commands)
    }

    fn scale_factor(&self) -> f64 {
        0.8
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        let mut subscriptions = vec![];

        let Page::Main(ref state) = self.page else {
            return Subscription::none();
        };

        subscriptions.push(subscription::unfold(
            "refresh_status".to_string(),
            state.request.clone(),
            refresh_status,
        ));

        match &self.servers {
            Some(servers) => {
                for server in servers.inner.values() {
                    subscriptions.push(subscription::unfold(
                        server.id.clone(),
                        (server.id.clone(), state.request.clone()),
                        refresh_output,
                    ))
                }
            }
            None => {}
        }

        Subscription::batch(subscriptions)
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        match &self.page {
            Page::Login(state) => self.login_page(state),
            Page::Main(state) => self.main_page(state),
        }
    }
}

impl App {
    fn main_page<'a>(&self, state: &MainState) -> Element<'a, Message, Renderer<Theme>> {
        let nav = navbar(Text::new(state.username.clone()).size(30).into());

        let mut col = Column::new().spacing(2);

        match &self.servers {
            Some(status) => {
                for server in status.inner.values() {
                    col = col.push(Card {
                        server_id: server.id.clone(),
                        status: server.running,
                        console: server.output.clone(),
                    });
                }
            }
            None => {}
        }

        column(vec![
            nav.into(),
            scrollable(col).height(Length::Fill).into(),
            status_bar(self.status.clone()).into(),
        ])
        .into()
    }

    fn login_page<'a>(&self, state: &LoginState) -> Element<'a, Message, Renderer<Theme>> {
        let nav = navbar(Text::new("").into());

        let handle = Handle::from_memory(LOGO);

        let logo = Image::new(handle).width(500.0);

        let input = |placeholder, value, field| {
            text_input(placeholder, value)
                .on_input(move |s| Message::UpdateLoginInput(field, s))
                .padding([10, 25])
                .size(24.0)
        };

        let address_input =
            input("Address", &state.address, LoginField::Address).width(Length::Fill);

        let username_input = input("Username", &state.username, LoginField::Username);
        let password_input = input("Password", &state.password, LoginField::Password);

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

        let user_col =
            column(vec![username_input.into(), password_input.into()]).width(Length::Fill);

        let user_row = row(vec![user_col.into(), login_button.into()]).height(100.0);

        column(vec![
            nav.into(),
            logo.into(),
            Space::new(Length::Fill, Length::Fill).into(),
            address_input.into(),
            Space::new(Length::Fill, 50.0).into(),
            user_row.into(),
            Space::new(Length::Fill, Length::Fill).into(),
            status_bar(self.status.clone()),
        ])
        .align_items(Alignment::Center)
        .into()
    }
}

async fn refresh_status(request: Request) -> (Message, Request) {
    tokio::time::sleep(Duration::from_secs(1)).await;

    let Some(global_status) = request.get_status().await else {
        return (
            Message::Error("Failed to retrieve status from remote".to_string()),
            request,
        );
    };

    (Message::StatusRefreshed(global_status), request)
}

async fn refresh_output(state: (String, Request)) -> (Message, (String, Request)) {
    tokio::time::sleep(Duration::from_secs(1)).await;

    let (server_id, request) = &state;

    let Some(ServerOutput {
        output: Some(output),
    }) = request.get_output(server_id.clone()).await
    else {
        return (Message::None, state);
    };

    (Message::OutputRefreshed(server_id.clone(), output), state)
}

fn main() {
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
        ..Default::default()
    });
}
