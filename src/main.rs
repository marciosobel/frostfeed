mod action;
mod api;
mod components;

use std::collections::HashMap;

use api::Feed;
use iced::{
    widget::{center, text},
    Element, Task,
};

mod screens;
use screens::{empty_view, feed};

use crate::action::Action;

fn main() -> iced::Result {
    iced::application(FrostFeed::new, FrostFeed::update, FrostFeed::view)
        .font(lucide_icons::LUCIDE_FONT_BYTES)
        .theme(FrostFeed::theme)
        .antialiasing(true)
        .centered()
        .run()
}

struct FrostFeed {
    theme: iced::Theme,
    feeds: HashMap<String, Feed>,
    screen: Screen,
    loading: bool,
}

enum Screen {
    EmptyView(empty_view::State),
    Feed(feed::State),
}

#[derive(Debug, Clone)]
enum Instruction {
    EmptyView(empty_view::Instruction),
    Feed(feed::Instruction),
}

#[derive(Debug, Clone)]
enum Message {
    // Screens
    EmptyView(empty_view::Message),
    Feed(feed::Message),

    FeedAdded(Feed),
}

impl FrostFeed {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                theme: iced::Theme::CatppuccinMocha,
                feeds: HashMap::new(),
                screen: Screen::EmptyView(empty_view::new()),
                loading: false,
            },
            Task::none(),
        )
    }

    fn theme(&self) -> iced::Theme {
        self.theme.clone()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::EmptyView(message) => {
                let Screen::EmptyView(screen) = &mut self.screen else {
                    return Task::none();
                };

                let action = screen
                    .update(message)
                    .map(Message::EmptyView)
                    .map_instruction(Instruction::EmptyView);
                return self.handle_action(action);
            }
            Message::Feed(message) => {
                let Screen::Feed(screen) = &mut self.screen else {
                    return Task::none();
                };

                let action = screen
                    .update(message)
                    .map(Message::Feed)
                    .map_instruction(Instruction::Feed);
                return self.handle_action(action);
            }
            Message::FeedAdded(feed) => {
                self.feeds.insert(feed.url.clone(), feed);
                self.loading = false;
                self.screen = Screen::Feed(feed::new());
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        if self.loading {
            return center(text("Loading, please wait...")).into();
        }

        match &self.screen {
            Screen::EmptyView(state) => state.view().map(Message::EmptyView),
            Screen::Feed(state) => state.view(self.theme(), &self.feeds).map(Message::Feed),
        }
    }

    fn handle_action(&mut self, action: Action<Instruction, Message>) -> Task<Message> {
        let instruction_task = match action.instruction {
            Some(instruction) => self.perform(instruction),
            None => Task::none(),
        };

        return instruction_task.chain(action.task);
    }

    fn perform(&mut self, instruction: Instruction) -> Task<Message> {
        match instruction {
            Instruction::EmptyView(instruction) => match instruction {
                empty_view::Instruction::AddFeed(url) => {
                    self.loading = true;
                    return Task::perform(api::Feed::from_url(url), Message::FeedAdded);
                }
            },
        }
    }
}
