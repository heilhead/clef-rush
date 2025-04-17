use {
    super::{App, Message},
    iced::{Element, Length, Subscription, Task, alignment, widget},
};

#[derive(Default)]
pub struct State {}

impl State {
    pub fn init(&mut self) -> Task<Message> {
        Task::none()
    }

    pub fn update(&mut self, _: Message) -> Task<Message> {
        Task::none()
    }

    pub fn view<'a>(&'a self, _: &'a App) -> Element<'a, Message> {
        widget::column![
            widget::vertical_space(),
            widget::text("Initializing..."),
            widget::vertical_space(),
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .into()
    }

    pub fn subscription<'a>(&'a self, _: &'a App) -> Subscription<Message> {
        Subscription::none()
    }
}
