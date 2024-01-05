mod request;

use iced::{
    executor,
    widget::{button, column, row, Container, Text},
    Application, Command, Element, Renderer, Settings, Theme,
};

use models::GlobalStatus;
use request::get_status;

use crate::request::start_server;

#[derive(Debug, Clone)]
enum Message {
    Refresh,
    Start,
    SetStatus(Option<GlobalStatus>),
    ServerStartResponse(bool),
}

struct MinecraftPanel {
    status: Option<GlobalStatus>,
}

impl Application for MinecraftPanel {
    type Executor = executor::Default;

    type Message = Message;

    type Theme = Theme;

    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            Self { status: None },
            Command::perform(get_status(), |global_status| {
                Message::SetStatus(global_status)
            }),
        )
    }

    fn title(&self) -> String {
        "Minecraft Panel".to_string()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        use Message::*;

        let mut commands = vec![];

        match message {
            SetStatus(status) => self.status = status,
            Refresh => {
                self.status = None;

                commands.push(Command::perform(get_status(), |global_status| {
                    Message::SetStatus(global_status)
                }));
            }
            Start => commands.push(Command::perform(
                start_server("minecraft".to_string()),
                |i| Message::ServerStartResponse(i),
            )),
            ServerStartResponse(_successful) => {}
        }

        Command::batch(commands)
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let refresh_button = button(Text::new("Refresh")).on_press(Message::Refresh);

        let start_button = button(Text::new("start")).on_press(Message::Start);

        let action_row = row(vec![refresh_button.into(), start_button.into()]).spacing(16);

        let status_text = match &self.status {
            Some(status) => Text::new(format!("{:?}", status)),
            None => Text::new("Loading status..."),
        };

        let main_column =
            Container::new(column(vec![action_row.into(), status_text.into()])).padding(64);

        main_column.into()
    }
}

fn main() {
    let _ = MinecraftPanel::run(Settings::default());
}
