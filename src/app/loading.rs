use {
    super::{App, Message},
    iced::{Element, Subscription, Task, widget},
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
        widget::column![widget::text("Loading...")].into()
    }

    pub fn subscription<'a>(&'a self, _: &'a App) -> Subscription<Message> {
        Subscription::none()
    }

    pub fn menu_view<'a>(&'a self, _: &'a App) -> Element<'a, Message> {
        widget::row![].into()
    }
}
