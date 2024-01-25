use std::time::Duration;

use models::ServerOutput;
use uuid::Uuid;

use crate::{
    components::{navbar, Card},
    request::Request,
    servers::Servers,
    theme, Element, Message, Page, LOGOUT_BUTTON, SETTINGS_BUTTON,
};

use iced::{
    subscription,
    widget::{button, column, image::Handle, row, scrollable, Column, Image, Text},
    Command, Length, Subscription,
};

#[derive(Debug, Clone)]
pub struct MainState {
    pub request: Request,
    pub username: String,

    pub token: Uuid,
    pub servers: Servers,
}

#[derive(Debug, Clone)]
pub enum Event {
    Super(Box<Message>),
    ToggleServer(String),
    SendCommand(String, String),
    StatusRefreshed(models::GlobalStatus),
    OutputRefreshed(String, Vec<String>),
    None,
}

impl MainState {
    pub fn update(&mut self, evt: Event) -> (Option<Message>, Command<Event>) {
        let mut msg = None;
        let mut commands = vec![];

        'm: {
            match evt {
                Event::Super(m) => msg = Some(*m),
                Event::StatusRefreshed(id) => self.servers.update(id),
                Event::OutputRefreshed(id, new) => {
                    if let Some(server) = self.servers.inner.get_mut(&id) {
                        server.output = new;
                    }
                }
                Event::SendCommand(id, command) => {
                    let request = self.request.clone();

                    let token = self.token.clone();

                    commands.push(Command::perform(
                        async move { request.send_command(id, command, token).await },
                        |_i| Event::None,
                    ))
                }
                Event::ToggleServer(server_id) => {
                    let request = self.request.clone();

                    let Some(server) = self.servers.inner.get(&server_id) else {
                        break 'm;
                    };

                    let token = self.token.clone();

                    match server.running {
                        true => commands.push(Command::perform(
                            async move { request.stop_server(server_id, token).await },
                            |_i| Event::None,
                        )),
                        false => commands.push(Command::perform(
                            async move { request.start_server(server_id, token).await },
                            |_i| Event::None,
                        )),
                    };
                }
                _ => {}
            }
        }

        (msg, Command::batch(commands))
    }

    pub fn view<'a>(&self) -> Element<'a, Event> {
        let username = Text::new(self.username.clone()).size(30);

        let settings_icon = Image::new(Handle::from_memory(SETTINGS_BUTTON));

        let settings_button = button(settings_icon)
            .style(theme::Button::Transparent)
            .on_press(Event::Super(Box::new(Message::GotoPage(Page::Settings))));

        let logout_icon = Image::new(Handle::from_memory(LOGOUT_BUTTON));

        let logout_button = button(logout_icon)
            .on_press(Event::Super(Box::new(Message::GotoPrevious)))
            .style(theme::Button::Transparent);

        let nav = navbar(
            row!(username, settings_button, logout_button)
                .spacing(24)
                .into(),
        );

        let mut col: Column<'a, Event, _> = Column::new().spacing(2);

        for server in self.servers.inner.values() {
            let id = server.id.clone();

            col = col.push(Element::from(Card {
                server_id: id.clone(),
                status: server.running,
                console: server.output.clone(),

                toggle: Event::ToggleServer(id.clone()),
                send: move |i| Event::SendCommand(id.clone(), i),
            }));
        }

        column!(nav, scrollable(col).height(Length::Fill))
            .height(Length::Fill)
            .into()
    }

    pub fn subscription(&self) -> Subscription<Event> {
        let mut subscriptions = vec![];

        subscriptions.push(subscription::unfold(
            "refresh_status".to_string(),
            (self.request.clone(), self.token.clone()),
            refresh_status,
        ));

        for server in self.servers.inner.values() {
            subscriptions.push(subscription::unfold(
                server.id.clone(),
                (server.id.clone(), self.request.clone(), self.token.clone()),
                refresh_output,
            ))
        }

        Subscription::batch(subscriptions)
    }
}

async fn refresh_status(state: (Request, Uuid)) -> (Event, (Request, Uuid)) {
    tokio::time::sleep(Duration::from_secs(1)).await;

    let (request, token) = &state;

    let Some(global_status) = request.get_status(token.clone()).await else {
        return (
            Event::Super(Box::new(Message::Error(
                "Failed to retrieve status from remote".to_string(),
            ))),
            state,
        );
    };

    (Event::StatusRefreshed(global_status), state)
}

async fn refresh_output(state: (String, Request, Uuid)) -> (Event, (String, Request, Uuid)) {
    tokio::time::sleep(Duration::from_secs(1)).await;

    let (server_id, request, token) = &state;

    let Some(ServerOutput {
        output: Some(output),
    }) = request.get_output(server_id.clone(), token.clone()).await
    else {
        return (Event::None, state);
    };

    (Event::OutputRefreshed(server_id.clone(), output), state)
}
