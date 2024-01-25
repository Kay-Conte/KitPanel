use iced::widget::text_input;

#[derive(Debug, Clone)]
pub struct TabNav {
    ordered: Vec<text_input::Id>,
    current: usize,
}

impl TabNav {
    pub fn new(ordered: Vec<text_input::Id>) -> Self {
        Self {
            ordered,
            current: 0,
        }
    }

    pub fn set(&mut self, id: text_input::Id) {
        if let Some(idx) = self.ordered.iter().position(|i| *i == id) {
            self.current = idx;
        }
    }

    pub fn next(&mut self) -> text_input::Id {
        self.current = (self.current + 1) % self.ordered.len();

        self.ordered[self.current].clone()
    }

    pub fn back(&mut self) -> text_input::Id {
        if self.current == 0 {
            self.current = self.ordered.len() - 1;
        } else {
            self.current = self.current - 1
        }

        self.ordered[self.current].clone()
    }
}
