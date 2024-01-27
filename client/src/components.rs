use iced::{
    widget::{
        button, column, component,
        image::Handle,
        row, scrollable,
        scrollable::{Direction, Properties},
        text_input, Button, Column, Component, Container, Image, Row, Space, Text,
    },
    Alignment, Length, Renderer,
};

use crate::{
    theme::{self, Theme},
    Element, Message, EXPAND_ARROW, EXPAND_ARROW_CLOSED, POWER_BUTTON,
};

pub fn icon_button<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
) -> Button<'a, M, Renderer<Theme>> {
    button(content).padding(10).style(theme::Button::Icon)
}

pub fn navbar<'a, M: 'a>(rhs: Element<'a, M>) -> Element<'a, M> {
    let title = Text::new("Kit Panel").size(30);

    row!(title, Space::new(Length::Fill, 0.0), rhs,)
        .align_items(Alignment::Center)
        .padding(20)
        .into()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Error(String),
    None,
}

pub fn status_bar<'a>(status: &'a Status) -> Element<'a, Message> {
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
        )
        .style(theme::Container::Secondary)
        .center_y()
        .height(50.0)
        .width(Length::Fill),

        Status::None => Container::new(Text::new(""))
            .style(theme::Container::Default)
            .height(50.0)
            .width(Length::Fill),
    }
    .into()
}

pub struct Card<M, F>
where
    F: Fn(String) -> M,
{
    pub server_id: String,
    pub status: bool,
    pub console: Vec<String>,

    pub toggle: M,
    pub send: F,
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

impl<M, F> Component<M, Renderer<Theme>> for Card<M, F>
where
    M: 'static + Clone,
    F: Fn(String) -> M,
{
    type State = CardState;

    type Event = CardMessage;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<M> {
        use CardMessage::*;

        match event {
            Expand => {
                state.expanded = !state.expanded;
                None
            }
            ToggleServer => Some(self.toggle.clone()),
            UpdateCommand(s) => {
                state.command = s;
                None
            }
            SubmitCommand => {
                let command = state.command.clone();

                state.command.clear();

                Some((self.send)(command))
            }
        }
    }

    fn view(&self, state: &Self::State) -> Element<'_, Self::Event> {
        let icon = Image::new(Handle::from_memory(POWER_BUTTON));

        let mut power_button = button(
            Container::new(icon)
                .height(Length::Fill)
                .width(Length::Fill)
                .center_x()
                .center_y(),
        )
        .width(Length::Fixed(75.0))
        .height(Length::Fill)
        .on_press(CardMessage::ToggleServer);

        if !self.status {
            power_button = power_button.style(theme::Button::Neutral);
        }

        let id: Element<'_, _> = Text::new(&self.server_id).size(30).into();

        let handle = Handle::from_memory(match state.expanded {
            true => EXPAND_ARROW,
            false => EXPAND_ARROW_CLOSED,
        });

        let icon = Container::new(Image::new(handle).height(32.0).width(32.0))
            .height(Length::Fill)
            .center_y();

        let status_row = row!(id, Space::new(Length::Fill, Length::Fill), icon,)
            .align_items(Alignment::Center)
            .padding([0, 20])
            .height(Length::Fill);

        let status_row = row!(
            power_button,
            button(status_row)
                .on_press(CardMessage::Expand)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(theme::Button::Transparent)
        )
        .height(Length::Fixed(75.0));

        let status_row = Container::new(status_row).style(theme::Container::Secondary);

        let mut col = Column::new().push(status_row);

        if state.expanded {
            let content: Vec<Element<'_, Self::Event>> = if self.console.len() == 0 {
                vec![Text::new("[KitPanel] No logs yet").size(20).into()]
            } else {
                self.console
                    .iter()
                    .rev()
                    .map(|i| Text::new(i).size(20).into())
                    .collect()
            };

            let scrollable = Container::new(
                scrollable(column(content))
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

impl<'a, M, F> From<Card<M, F>> for Element<'a, M>
where
    M: 'static + Clone,
    F: 'static + Fn(String) -> M,
{
    fn from(value: Card<M, F>) -> Self {
        component(value)
    }
}

pub fn tab_bar<'a>(
    options: Vec<String>,
    selected: String,
    on_change: Option<Message>,
) -> Element<'a, Message> {
    let mut row = Row::new().spacing(75);

    for option in options.iter() {
        let mut text = Text::new(option.clone()).size(30);

        if *option != selected {
            text = text.style(theme::Text::Hint);
        }

        let button = button(text)
            .style(theme::Button::Icon)
            .on_press_maybe(on_change.clone());

        row = row.push(button);
    }

    row.into()
}
