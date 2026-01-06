mod action;
mod api;
mod components;

use std::collections::HashMap;

use api::Feed;
use iced::{
    widget::{center, text},
    Element, Task, Theme,
};

mod screens;

use crate::{
    action::Action,
    screens::{empty_view, feed, settings},
};

fn main() -> iced::Result {
    iced::application(FrostFeed::new, FrostFeed::update, FrostFeed::view)
        .font(lucide_icons::LUCIDE_FONT_BYTES)
        .theme(FrostFeed::theme)
        .antialiasing(true)
        .centered()
        .run()
}

struct FrostFeed {
    settings: Settings,
    screen: Screen,
    loading: bool,
}

#[derive(Debug, Clone)]
enum Screen {
    EmptyView(empty_view::State),
    Feed(feed::State),
    Settings(settings::State),
}

#[derive(Debug, Clone)]
enum Instruction {
    EmptyView(empty_view::Instruction),
    Feed(feed::Instruction),
    Settings(settings::Instruction),
}

#[derive(Debug, Clone)]
enum Message {
    // Screens
    EmptyView(empty_view::Message),
    Feed(feed::Message),
    Settings(settings::Message),

    FeedAdded(Feed),
    ChangeScreen(Screen),
}

impl FrostFeed {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                settings: Settings::new(),
                screen: Screen::EmptyView(empty_view::new()),
                loading: false,
            },
            Task::none(),
        )
    }

    fn theme(&self) -> iced::Theme {
        self.settings.theme.clone()
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
            Message::Settings(message) => {
                let Screen::Settings(screen) = &mut self.screen else {
                    return Task::none();
                };

                let action = screen
                    .update(&mut self.settings, message)
                    .map(Message::Settings)
                    .map_instruction(Instruction::Settings);
                return self.handle_action(action);
            }
            Message::FeedAdded(feed) => {
                println!("Feed added: {}", feed.title);
                self.settings.feeds.insert(feed.url.clone(), feed);
                self.loading = false;
                if let Screen::EmptyView(_) = self.screen {
                    return Task::done(Message::ChangeScreen(Screen::Feed(feed::new())));
                }
            }
            Message::ChangeScreen(screen) => self.screen = screen,
        }

        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        if self.loading {
            return center(text("Loading, please wait...")).into();
        }

        match &self.screen {
            Screen::EmptyView(state) => state.view().map(Message::EmptyView),
            Screen::Feed(state) => state
                .view(self.theme(), &self.settings.feeds)
                .map(Message::Feed),
            Screen::Settings(state) => state.view(&self.settings).map(Message::Settings),
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
            Instruction::Feed(instruction) => match instruction {
                feed::Instruction::OpenSettings => {
                    Task::done(Message::ChangeScreen(Screen::Settings(settings::new())))
                }
            },
            Instruction::Settings(instruction) => match instruction {
                settings::Instruction::AddFeed(url) => {
                    Task::perform(api::Feed::from_url(url), Message::FeedAdded)
                }
                settings::Instruction::GoBack => {
                    let screen = if self.settings.feeds.is_empty() {
                        Screen::EmptyView(empty_view::new())
                    } else {
                        Screen::Feed(feed::new())
                    };

                    Task::done(Message::ChangeScreen(screen))
                }
            },
        }
    }
}

struct Settings {
    feeds: HashMap<String, Feed>,
    theme: Theme,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            theme: Theme::CatppuccinMocha,
            feeds: HashMap::new(),
        }
    }
}
