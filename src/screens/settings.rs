use std::collections::hash_map::Values;

use iced::{
    widget::{button, center, column, container, row, space, stack, text, text_input},
    Alignment, Element, Font, Length,
};
use lucide_icons::iced::{icon_plus, icon_trash_2, icon_x};

use crate::{action::Action, api::Feed, Settings};

#[derive(Debug, Clone)]
pub struct State {
    new_feed_url: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddFeed,
    FeedUrlChanged(String),
    RemoveFeed(String),
    ExitScreen,
}

#[derive(Debug, Clone)]
pub enum Instruction {
    AddFeed(String),
    GoBack,
}

pub fn new() -> State {
    State {
        new_feed_url: String::new(),
    }
}

impl State {
    pub fn update<'a>(
        &'a mut self,
        settings: &'a mut Settings,
        message: Message,
    ) -> Action<Instruction, Message> {
        match message {
            Message::ExitScreen => return Action::instruction(Instruction::GoBack),
            Message::FeedUrlChanged(url) => self.new_feed_url = url,
            Message::AddFeed => {
                let url = self.new_feed_url.clone();
                self.new_feed_url = String::new();
                return Action::instruction(Instruction::AddFeed(url));
            }
            Message::RemoveFeed(url) => {
                println!("Removing {}", url);
                settings.feeds.remove(&url);
                println!("Removed {}", url);
            }
        }

        Action::none()
    }

    pub fn view<'a>(&'a self, settings: &'a Settings) -> Element<'a, Message> {
        let header = self.header();
        let feeds = center(self.feeds(settings.feeds.values(), settings.feeds.is_empty()));

        container(column![header, feeds])
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .into()
    }

    pub fn header(&self) -> Element<'_, Message> {
        let go_back_button =
            button(row![icon_x(), text("Close")].spacing(10)).on_press(Message::ExitScreen);

        stack![
            row![space::horizontal(), go_back_button],
            center(text("Settings")),
        ]
        .into()
    }

    pub fn feeds<'a>(
        &'a self,
        feeds: Values<'a, String, Feed>,
        is_empty: bool,
    ) -> Element<'a, Message> {
        let mut add_button = row![].align_y(Alignment::Center).spacing(10);

        let mut add_feed = column![
            text("Add a new feed"),
            text_input("Insert URL here", &self.new_feed_url)
                .on_input(Message::FeedUrlChanged)
                .on_submit(Message::AddFeed),
        ]
        .spacing(10);

        if !is_empty {
            add_button = add_button.push(text("Current feeds:").size(20).font(Font {
                weight: iced::font::Weight::Semibold,
                ..Font::DEFAULT
            }));
        }

        add_button = add_button
            .push(space::horizontal())
            .push(button(row![icon_plus(), text("Add")]).on_press(Message::AddFeed));

        add_feed = add_feed.push(add_button);
        if !is_empty {
            for feed in feeds {
                let feed_text = row![
                    button(icon_trash_2())
                        .on_press_with(|| Message::RemoveFeed(feed.url.clone()))
                        .style(button::danger),
                    text!("{} ({})", feed.title, feed.url),
                ]
                .spacing(10);
                add_feed = add_feed.push(feed_text);
            }
        }

        add_feed.into()
    }
}
