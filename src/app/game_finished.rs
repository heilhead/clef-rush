use {
    super::{App, GameResults, Message, StateTransition},
    iced::{Element, Subscription, Task, widget},
};

pub struct State {
    results: GameResults,
}

impl State {
    pub fn new(results: GameResults) -> Self {
        Self { results }
    }

    pub fn init(&mut self) -> Task<Message> {
        Task::none()
    }

    pub fn update(&mut self, _: Message) -> Task<Message> {
        Task::none()
    }

    pub fn view<'a>(&'a self, _: &'a App) -> Element<'a, Message> {
        widget::column![
            widget::button("Play Again").on_press(Message::StateTransition(
                StateTransition::GameActive(self.results.settings.clone())
            )),
            widget::button("Main Menu")
                .on_press(Message::StateTransition(StateTransition::MainMenu))
        ]
        .into()
    }

    pub fn subscription<'a>(&'a self, _: &'a App) -> Subscription<Message> {
        Subscription::none()
    }
}
