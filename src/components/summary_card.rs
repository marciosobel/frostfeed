use crate::api::{Feed, FeedItem};
use iced::{
    font::Weight,
    widget::{button, column, container, text},
    Element, Font, Length,
};

pub struct SummaryCard<'a, Message> {
    element: Element<'a, Message>,
    on_click: Option<Box<dyn Fn(FeedItem) -> Message + 'a>>,
    item: &'a FeedItem,
    selected: bool,
}

pub fn summary_card<'a, Message>(feed: &'a Feed, item: &'a FeedItem) -> SummaryCard<'a, Message>
where
    Message: Clone + 'a,
{
    let header = text(&feed.title)
        .width(Length::Fill)
        .wrapping(text::Wrapping::Word)
        .size(14)
        .font(Font {
            weight: Weight::Bold,
            ..Font::DEFAULT
        });

    let element = column![header, text(&item.title).size(16)]
        .spacing(10)
        .into();

    SummaryCard {
        element,
        item,
        on_click: None,
        selected: false,
    }
}

impl<'a, Message> SummaryCard<'a, Message> {
    pub fn on_press(mut self, message: impl Fn(FeedItem) -> Message + 'a) -> Self {
        self.on_click = Some(Box::new(message));
        self
    }

    pub fn selected(mut self, state: bool) -> Self {
        self.selected = state;
        self
    }
}

impl<'a, Message> Into<Element<'a, Message>> for SummaryCard<'a, Message>
where
    Message: Clone + 'a,
{
    fn into(self) -> Element<'a, Message> {
        let container = container(self.element)
            .style(move |theme: &iced::Theme| {
                let palette = theme.extended_palette();
                let style = container::Style::default();
                let pair = if self.selected {
                    palette.background.strongest
                } else {
                    palette.background.weaker
                };
                style.background(pair.color).color(pair.text)
            })
            .padding(10)
            .into();

        match self.on_click {
            Some(msg) => button(container)
                .on_press(msg(self.item.clone()))
                .padding(0)
                .style(button::text)
                .into(),
            _ => container,
        }
    }
}
