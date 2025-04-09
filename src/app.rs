use {
    crate::{
        input::{self, PortDescriptor},
        verovio,
    },
    derive_more::From,
    iced::{Element, Subscription, Task, Theme},
    midly::MidiMessage,
};

mod game_active;
mod game_finished;
mod loading;
mod main_menu;

const USE_MOCK_INPUT: bool = true;

#[derive(Debug, Clone)]
pub struct GameSettings {
    input_port: input::PortDescriptor,
}

#[derive(Debug, Clone)]
pub struct GameResults {
    settings: GameSettings,
}

#[derive(Debug, Clone)]
pub enum StateTransition {
    MainMenu,
    GameActive(GameSettings),
    GameFinished(GameResults),
}

enum State {
    Loading(loading::State),
    MainMenu(main_menu::State),
    GameActive(game_active::State),
    GameFinished(game_finished::State),
}

impl State {
    fn init(&mut self) -> Task<Message> {
        match self {
            Self::Loading(state) => state.init(),
            Self::MainMenu(state) => state.init(),
            Self::GameActive(state) => state.init(),
            Self::GameFinished(state) => state.init(),
        }
    }
}

pub struct App {
    state: State,
}

#[derive(From, Debug, Clone)]
pub enum Message {
    StateTransition(StateTransition),
    SelectInputPort(PortDescriptor),
    RefreshDeviceList,
    InputEvent(#[from] MidiMessage),
    InputWorkerReady(input::Connector),
    Ready,
}

impl App {
    pub fn boot() -> (Self, Task<Message>) {
        (
            Self {
                state: State::Loading(Default::default()),
            },
            Task::future(verovio::initialize())
                .map(|_| Message::StateTransition(StateTransition::MainMenu)),
        )
    }

    pub fn update(&mut self, event: Message) -> Task<Message> {
        match event {
            Message::StateTransition(new_state) => {
                match new_state {
                    StateTransition::MainMenu => {
                        self.state = State::MainMenu(main_menu::State::new());
                    }

                    StateTransition::GameActive(settings) => {
                        self.state = State::GameActive(game_active::State::new(settings));
                    }

                    StateTransition::GameFinished(results) => {
                        self.state = State::GameFinished(game_finished::State::new(results));
                    }
                }

                return self.state.init();
            }

            event => {
                return match &mut self.state {
                    State::Loading(state) => state.update(event),
                    State::MainMenu(state) => state.update(event),
                    State::GameActive(state) => state.update(event),
                    State::GameFinished(state) => state.update(event),
                };
            }
        }
    }

    pub fn view(&self) -> Element<Message> {
        // TODO: Header/footer, global layout.

        match &self.state {
            State::Loading(state) => state.view(self),
            State::MainMenu(state) => state.view(self),
            State::GameActive(state) => state.view(self),
            State::GameFinished(state) => state.view(self),
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match &self.state {
            State::Loading(state) => state.subscription(self),
            State::MainMenu(state) => state.subscription(self),
            State::GameActive(state) => state.subscription(self),
            State::GameFinished(state) => state.subscription(self),
        }
    }

    pub fn theme(&self) -> Theme {
        Theme::Light
    }
}
