mod incoming;

use iced::{
    executor,
    widget::{button, column, row, Text},
    Application, Command, Element, Renderer, Settings, Theme,
};
use incoming::GlobalStatus;

#[derive(Debug, Clone)]
enum Message {
    Refresh,
    SetStatus(GlobalStatus),
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
            Command::perform(get_status(), |global_status: GlobalStatus| {
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
            SetStatus(status) => self.status = Some(status),
            Refresh => {
                commands.push(Command::perform(
                    get_status(),
                    |global_status: GlobalStatus| Message::SetStatus(global_status),
                ));
            }
        }

        Command::batch(commands)
    }

    fn view(&self) -> Element<'_, Self::Message, Renderer<Self::Theme>> {
        let refresh_button = button(Text::new("Refresh")).on_press(Message::Refresh);

        let action_row = row(vec![refresh_button.into()]);

        let status_text = match &self.status {
            Some(status) => Text::new(format!("{:?}", status)),
            None => Text::new("Loading status..."),
        };

        let main_column = column(vec![action_row.into(), status_text.into()]);

        main_column.into()
    }
}

async fn get_status() -> GlobalStatus {
    todo!("implement refresh");
}

fn main() {
    let _ = MinecraftPanel::run(Settings::default());
}
