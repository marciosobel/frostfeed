use iced::{
    widget::{button, center, column, row, space, text, text_input},
    Alignment, Element,
};
use lucide_icons::iced::icon_plus;

use crate::action::Action;

#[derive(Debug, Clone)]
pub struct State {
    feed_url: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    FeedUrlChanged(String),
    AddFeed,
}

impl Default for State {
    fn default() -> Self {
        Self {
            feed_url: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    AddFeed(String),
}

impl State {
    pub fn view(&self) -> Element<'_, Message> {
        let prompt = text("No feeds yet. Add a new feed below").size(18);
        let add_maybe = if self.has_valid_url() {
            Some(Message::AddFeed)
        } else {
            None
        };

        let input = text_input("Insert the feed URL here!", &self.feed_url)
            .on_input(Message::FeedUrlChanged)
            .on_submit_maybe(add_maybe.clone());

        let add = button(
            row![icon_plus(), text("Add")]
                .spacing(10)
                .align_y(Alignment::Center),
        )
        .on_press_maybe(add_maybe);

        center(
            column![prompt, input, row![space::horizontal(), add]]
                .align_x(Alignment::Center)
                .spacing(10),
        )
        .padding(10)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Action<Instruction, Message> {
        match message {
            Message::FeedUrlChanged(text) => self.feed_url = text,
            Message::AddFeed => {
                return Action::instruction(Instruction::AddFeed(self.feed_url.clone()))
            }
        }

        Action::none()
    }

    fn has_valid_url(&self) -> bool {
        let url = &self.feed_url;
        url.len() > 0 && (url.starts_with("http://") || url.starts_with("https://"))
    }
}

pub fn new() -> State {
    State::default()
}
