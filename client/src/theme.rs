use iced::{
    application,
    widget::{
        button, container,
        rule::{self, FillMode},
        scrollable::{self, Scroller},
        text, text_input, toggler,
    },
    Color, Vector,
};

use iced_hex_color::hex_color;

fn darken(mut base: Color, factor: f32) -> Color {
    let f = |i: f32| {
        let diff = i * factor;

        i - diff
    };

    base.r = f(base.r);
    base.g = f(base.g);
    base.b = f(base.b);

    base
}

pub struct Palette {
    base: Color,
    secondary: Color,
    active: Color,
    destructive: Color,
    neutral: Color,
    hint: Color,
    value: Color,
}

impl Palette {
    fn dark() -> Palette {
        Palette {
            base: hex_color!(#303036),
            secondary: hex_color!(#393940),
            active: hex_color!(#617855),
            destructive: hex_color!(#8d4839),
            neutral: hex_color!(#54545A),
            hint: hex_color!(#B5B9C3),
            value: Color::WHITE,
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette::dark()
    }
}

#[derive(Default)]
pub struct Theme {
    pub palette: Palette,
}

#[derive(Default, Clone)]
pub enum Application {
    #[default]
    Default,
}

impl application::StyleSheet for Theme {
    type Style = Application;

    fn appearance(&self, _style: &Self::Style) -> application::Appearance {
        application::Appearance {
            background_color: self.palette.base,
            text_color: self.palette.value,
        }
    }
}

#[derive(Default, Clone)]
pub enum Text {
    #[default]
    Default,
    Hint,
}

impl text::StyleSheet for Theme {
    type Style = Text;

    fn appearance(&self, style: Self::Style) -> text::Appearance {
        let color = match style {
            Text::Default => Color::WHITE,
            Text::Hint => self.palette.hint,
        };

        text::Appearance { color: Some(color) }
    }
}

#[derive(Default, Clone)]
pub enum Container {
    #[default]
    Default,
    Secondary,
    Destructive,
}

impl container::StyleSheet for Theme {
    type Style = Container;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        let color = match style {
            Container::Default => Color::TRANSPARENT,
            Container::Secondary => self.palette.secondary,
            Container::Destructive => self.palette.destructive,
        };

        container::Appearance {
            text_color: None,
            background: Some(color.into()),
            border_radius: 0.0.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

#[derive(Default, Clone, PartialEq)]
pub enum Button {
    #[default]
    Active,
    Destructive,
    Neutral,
    Icon,
    Transparent,
}

impl button::StyleSheet for Theme {
    type Style = Button;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        let color = match style {
            Button::Active => self.palette.active,
            Button::Destructive => self.palette.destructive,
            Button::Neutral => self.palette.neutral,
            Button::Icon | Button::Transparent => Color::TRANSPARENT,
        };

        let radius = match style {
            Button::Icon => f32::MAX,
            _ => 0.0,
        };

        button::Appearance {
            shadow_offset: Vector::new(0.0, 0.0),
            background: Some(color.into()),
            border_radius: radius.into(),
            border_width: 0.0,
            border_color: Color::WHITE,
            text_color: Color::WHITE,
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let mut appearance = self.active(style);

        appearance.background = Some(
            match style {
                Button::Active => darken(self.palette.active, 0.10),
                Button::Destructive => darken(self.palette.destructive, 0.10),
                Button::Neutral => darken(self.palette.neutral, 0.10),
                Button::Icon | Button::Transparent => Color::from_rgba8(10, 10, 10, 0.1),
            }
            .into(),
        );

        appearance
    }
}

#[derive(Default, Clone)]
pub enum Scrollable {
    #[default]
    Default,
}

impl scrollable::StyleSheet for Theme {
    type Style = Scrollable;

    fn active(&self, _style: &Self::Style) -> scrollable::Scrollbar {
        scrollable::Scrollbar {
            background: None,
            border_radius: 0.0.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            scroller: Scroller {
                color: Color::TRANSPARENT,
                border_radius: 0.0.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }

    fn hovered(
        &self,
        _style: &Self::Style,
        _is_mouse_over_scrollbar: bool,
    ) -> scrollable::Scrollbar {
        scrollable::Scrollbar {
            background: None,
            border_radius: 0.0.into(),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            scroller: Scroller {
                color: Color::TRANSPARENT,
                border_radius: 0.0.into(),
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            },
        }
    }
}

#[derive(Default, Clone)]
pub enum TextInput {
    #[default]
    Default,
}

impl text_input::StyleSheet for Theme {
    type Style = TextInput;

    fn active(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: self.palette.secondary.into(),
            border_width: 0.0,
            border_radius: 0.0.into(),
            border_color: Color::TRANSPARENT,
            icon_color: Color::WHITE,
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        self.active(style)
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        self.palette.hint
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        Color::WHITE
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        self.palette.hint
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba8(155, 155, 155, 0.2)
    }

    fn disabled(&self, style: &Self::Style) -> text_input::Appearance {
        self.active(style)
    }
}

#[derive(Default, Clone)]
pub enum Toggler {
    #[default]
    Default,
}

impl toggler::StyleSheet for Theme {
    type Style = Toggler;

    fn active(&self, _style: &Self::Style, is_active: bool) -> toggler::Appearance {
        let color = if is_active {
            self.palette.active
        } else {
            self.palette.neutral
        };

        toggler::Appearance {
            background: color,
            background_border: None,
            foreground: self.palette.base,
            foreground_border: None,
        }
    }

    fn hovered(&self, style: &Self::Style, is_active: bool) -> toggler::Appearance {
        self.active(style, is_active)
    }
}

#[derive(Default, Clone)]
pub enum Rule {
    #[default]
    Default,
}

impl rule::StyleSheet for Theme {
    type Style = Rule;

    fn appearance(&self, _style: &Self::Style) -> rule::Appearance {
        rule::Appearance {
            color: self.palette.value,
            width: 1,
            radius: 0.0.into(),
            fill_mode: FillMode::Full,
        }
    }
}
