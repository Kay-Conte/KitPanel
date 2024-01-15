use iced::{
    widget::{
        button, column, component,
        image::Handle,
        row, scrollable,
        scrollable::{Direction, Properties},
        text_input, Column, Component, Container, Image, Space, Text,
    },
    Alignment, Element, Length, Renderer,
};

use crate::{
    theme::{self, Theme},
    Message, EXPAND_ARROW, EXPAND_ARROW_CLOSED, POWER_BUTTON,
};

pub fn navbar<'a>(rhs: Element<'a, Message, Renderer<Theme>>) -> Element<'a, Message, Renderer<Theme>> {
    let title = Text::new("Kit Panel").size(30);

    row(vec![
        title.into(),
        Space::new(Length::Fill, 0.0).into(),
        rhs.into(),
    ])
    .align_items(Alignment::Center)
    .padding(20)
    .into()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Error(String),
    None,
}

pub fn status_bar<'a>(status: Status) -> Element<'a, Message, Renderer<Theme>> {
    match status {
        Status::Error(s) => Container::new(
            row![
                Container::<'a, Message, Renderer<Theme>>::new(Text::new(""))
                    .height(Length::Fill)
                    .width(25.0)
                    .style(theme::Container::Destructive),
                Text::new(s).size(20).style(theme::Text::Hint)
            ]
            .spacing(10)
            .align_items(Alignment::Center),
        ).style(theme::Container::Secondary)
        .center_y()
        .height(50.0)
        .width(Length::Fill),

        Status::None => Container::new(Text::new(""))
            .style(theme::Container::Secondary)
            .height(50.0)
            .width(Length::Fill),
    }
    .into()
}

pub struct Card {
    pub server_id: String,
    pub status: bool,
    pub console: Vec<String>,
}

pub struct CardState {
    expanded: bool,
    command: String,
}

impl Default for CardState {
    fn default() -> Self {
        Self {
            expanded: false,
            command: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum CardMessage {
    Expand,
    ToggleServer,
    UpdateCommand(String),
    SubmitCommand,
}

impl Component<Message, Renderer<Theme>> for Card {
    type State = CardState;

    type Event = CardMessage;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        use CardMessage::*;

        match event {
            Expand => {
                state.expanded = !state.expanded;
                None
            }
            ToggleServer => Some(Message::ToggleServer(self.server_id.clone())),
            UpdateCommand(s) => {
                state.command = s;
                None
            }
            SubmitCommand => {
                let command = state.command.clone();

                state.command.clear();

                Some(Message::SendCommand(
                    self.server_id.clone(),
                    format!("{}\n", command),
                ))
            }
        }
    }

    fn view(&self, state: &Self::State) -> Element<'_, Self::Event, Renderer<Theme>> {
        let icon = Image::new(Handle::from_memory(POWER_BUTTON));

        let status = match self.status {
            true => button(
                Container::new(icon)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .center_x()
                    .center_y(),
            ),
            false => button(
                Container::new(icon)
                    .height(Length::Fill)
                    .width(Length::Fill)
                    .center_x()
                    .center_y(),
            )
            .style(theme::Button::Neutral),
        }
        .width(Length::Fixed(75.0))
        .height(Length::Fill)
        .on_press(CardMessage::ToggleServer);

        let id: Element<'_, _, Renderer<Theme>> = Text::new(&self.server_id).size(30).into();

        let handle = Handle::from_memory(match state.expanded {
            true => EXPAND_ARROW,
            false => EXPAND_ARROW_CLOSED,
        });

        let icon = Container::new(Image::new(handle).height(32.0).width(32.0))
            .height(Length::Fill)
            .center_y();

        let status_row = row(vec![
            id.into(),
            Space::new(Length::Fill, Length::Fill).into(),
            icon.into(),
        ])
        .align_items(Alignment::Center)
        .padding([0, 20])
        .height(Length::Fill);

        let status_row = Container::new(
            row(vec![
                status.into(),
                button(status_row)
                    .on_press(CardMessage::Expand)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(theme::Button::Transparent)
                    .into(),
            ])
            .height(Length::Fixed(75.0)),
        )
        .style(theme::Container::Secondary);

        let mut col = Column::new().push(status_row);

        if state.expanded {
            let scrollable = Container::new(
                scrollable(column(
                    self.console
                        .iter()
                        .rev()
                        .map(|i| Text::new(i).size(20).into())
                        .collect(),
                ))
                .direction(Direction::Vertical(
                    Properties::new().alignment(scrollable::Alignment::End),
                ))
                .height(Length::Fixed(450.0))
                .width(Length::Fill),
            )
            .padding(15);

            let console_col = column(vec![
                scrollable.into(),
                text_input("Enter a command", &state.command)
                    .on_input(CardMessage::UpdateCommand)
                    .on_submit(CardMessage::SubmitCommand)
                    .size(20)
                    .into(),
                Space::new(0.0, Length::Fixed(30.0)).into(),
            ]);

            col = col.push(Container::new(column(vec![console_col.into()])));
        }

        col.into()
    }
}

impl<'a> From<Card> for Element<'a, Message, Renderer<Theme>> {
    fn from(value: Card) -> Self {
        component(value)
    }
}
