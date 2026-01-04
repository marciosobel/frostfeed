use std::collections::HashMap;

use iced::{
    font::{self, Weight},
    time::milliseconds,
    widget::{
        button, center, center_x, column, image, markdown, operation::AbsoluteOffset, row,
        scrollable, sensor, space, text, text_input, Id,
    },
    Alignment, Element, Font, Function, Length, Task,
};
use lucide_icons::iced::{
    icon_panel_left_close, icon_panel_left_open, icon_search, icon_settings_2,
};

use crate::{
    action::Action,
    api::{self, Feed, FeedItem},
    components,
};

pub struct State {
    focused_item: Option<FocusedItem>,
    show_sidebar: bool,
    search: String,
}

enum Image {
    Ready(image::Handle),
    Loading,
}

struct FocusedItem {
    inner: FeedItem,
    images: HashMap<markdown::Uri, Image>,
    markdown: Vec<markdown::Item>,
}

#[derive(Debug, Clone)]
pub enum Message {
    FocusItem(FeedItem),
    HideSidebar,
    ShowSidebar,
    MarkdownLinkClicked(markdown::Uri),
    ImageShown(String),
    ImageDownloaded(String, image::Handle),
    SearchChanged(String),
    Search,
    OpenSettings,
}

#[derive(Debug, Clone)]
pub enum Instruction {}

pub fn new() -> State {
    State::default()
}

impl Default for State {
    fn default() -> Self {
        Self {
            focused_item: None,
            show_sidebar: true,
            search: String::new(),
        }
    }
}

impl State {
    const MAIN_CONTENT_SCROLL_ID: &str = "MAIN_CONTENT_SCROLL_ID";

    pub fn view<'a>(
        &'a self,
        theme: iced::Theme,
        feeds: &'a HashMap<String, Feed>,
    ) -> Element<'a, Message> {
        column![self.header(), self.main(theme, feeds),]
            .padding(10)
            .spacing(10)
            .into()
    }

    fn main<'a>(
        &'a self,
        theme: iced::Theme,
        feeds: &'a HashMap<String, Feed>,
    ) -> Element<'a, Message> {
        let feed_content = if let Some(item) = &self.focused_item {
            scrollable(
                column![
                    text(&item.inner.title).size(36).font(Font {
                        weight: Weight::Bold,
                        ..Font::DEFAULT
                    }),
                    item.view(theme)
                ]
                .spacing(10),
            )
            .id(Id::new(Self::MAIN_CONTENT_SCROLL_ID))
            .width(Length::Fill)
            .spacing(10)
            .into()
        } else {
            center(text("Select some content to view.")).into()
        };

        if self.show_sidebar {
            row![self.sidebar(feeds), feed_content].spacing(10).into()
        } else {
            feed_content
        }
    }

    pub fn update(&mut self, message: Message) -> Action<Instruction, Message> {
        match message {
            Message::FocusItem(item) => {
                self.focused_item = Some(FocusedItem::new(item));
                let task = iced::widget::operation::scroll_to(
                    Self::MAIN_CONTENT_SCROLL_ID,
                    AbsoluteOffset { y: 0.0, x: 0.0 },
                );
                return Action::task(task);
            }
            Message::HideSidebar => self.show_sidebar = false,
            Message::ShowSidebar => self.show_sidebar = true,
            Message::MarkdownLinkClicked(uri) => _ = open::that(uri),
            Message::ImageShown(url) => {
                let Some(item) = &mut self.focused_item else {
                    return Action::none();
                };

                if item.images.contains_key(&url) {
                    return Action::none();
                }

                _ = item.images.insert(url.clone(), Image::Loading);

                let task = Task::perform(
                    api::download_image(url.clone()),
                    Message::ImageDownloaded.with(url),
                );
                return Action::task(task);
            }
            Message::ImageDownloaded(url, handle) => {
                let Some(item) = &mut self.focused_item else {
                    return Action::none();
                };

                let _ = item.images.insert(url, Image::Ready(handle));
            }
            Message::SearchChanged(text) => self.search = text,
            Message::Search => todo!(),
            Message::OpenSettings => todo!(),
        }

        Action::none()
    }

    fn sidebar<'a>(&'a self, feeds: &'a HashMap<String, Feed>) -> Element<'a, Message> {
        let mut cards = column![].spacing(10);

        for feed in feeds.values() {
            for item in &feed.items {
                let selected = if let Some(focused) = &self.focused_item {
                    focused.inner.url == item.url
                } else {
                    false
                };
                let card = components::summary_card(feed, item)
                    .on_press(Message::FocusItem)
                    .selected(selected);
                cards = cards.push(card);
            }
        }

        scrollable(column![cards].spacing(10))
            .width(Length::Fixed(350.0))
            .height(Length::Fill)
            .spacing(0)
            .into()
    }

    fn header(&self) -> Element<'_, Message> {
        let toggle_sidebar = {
            let (element, message) = if self.show_sidebar {
                (icon_panel_left_close(), Message::HideSidebar)
            } else {
                (icon_panel_left_open(), Message::ShowSidebar)
            };

            button(element).on_press(message)
        };

        let search = text_input("Search", &self.search)
            .on_input(Message::SearchChanged)
            .on_submit(Message::Search)
            .icon(text_input::Icon {
                font: Font {
                    family: font::Family::Name("lucide"),
                    ..Font::DEFAULT
                },
                side: text_input::Side::Left,
                size: None,
                spacing: 10.0,
                code_point: lucide_icons::Icon::Search.into(),
            });

        let settings = button(
            row![icon_settings_2(), text("Settings")]
                .spacing(10)
                .align_y(Alignment::Center),
        )
        .on_press(Message::OpenSettings);

        row![
            toggle_sidebar,
            space::horizontal(),
            search,
            space::horizontal(),
            settings,
        ]
        .into()
    }
}

impl FocusedItem {
    fn new(item: FeedItem) -> Self {
        let content;
        match &item.description {
            Some(t) => {
                let read = html2text::from_read(t.as_bytes(), 999).unwrap();
                content = read.replace("  ", " ").trim().to_string();
            }
            None => content = "Unable to get content".to_string(),
        };

        let markdown: Vec<_> = markdown::parse(&content).collect();

        Self {
            inner: item,
            markdown,
            images: HashMap::new(),
        }
    }

    fn view(&self, theme: iced::Theme) -> Element<'_, Message> {
        markdown::view_with(&self.markdown, theme, self).into()
    }
}

impl<'a> markdown::Viewer<'a, Message> for FocusedItem {
    fn on_link_click(url: markdown::Uri) -> Message {
        Message::MarkdownLinkClicked(url)
    }

    fn image(
        &self,
        _settings: markdown::Settings,
        url: &'a markdown::Uri,
        _title: &'a str,
        _alt: &markdown::Text,
    ) -> Element<'a, Message> {
        if let Some(Image::Ready(handle)) = &self.images.get(url) {
            center_x(image(handle)).into()
        } else {
            sensor(text("Loading"))
                .key_ref(url.as_str())
                .delay(milliseconds(500))
                .on_show(|_| Message::ImageShown(url.clone()))
                .into()
        }
    }
}
