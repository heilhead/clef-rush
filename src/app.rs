use {
    crate::{
        input::{self, PortDescriptor},
        verovio,
    },
    derive_more::From,
    iced::{
        Color,
        Element,
        Length,
        Subscription,
        Task,
        Theme,
        alignment::Vertical,
        font,
        widget::{self, text::Shaping},
    },
    midly::MidiMessage,
};

mod game_active;
mod game_finished;
mod loading;
mod main_menu;

const USE_MOCK_INPUT: bool = true;
const TITLE: &str = "Clef Rush";

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
    AdvanceChallenge,
    UpdateChallengeHint(widget::svg::Handle),
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
        let title = widget::text(TITLE)
            .size(36)
            .font(Font::Title)
            .shaping(Shaping::Advanced);

        let header = widget::row![title].spacing(20).align_y(Vertical::Center);

        let content = match &self.state {
            State::Loading(state) => state.view(self),
            State::MainMenu(state) => state.view(self),
            State::GameActive(state) => state.view(self),
            State::GameFinished(state) => state.view(self),
        };

        let res: Element<_> = widget::column![header, content]
            .spacing(10)
            .padding(20)
            .height(Length::Fill)
            .width(Length::Fill)
            .into();

        res.explain(Color::BLACK)
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
        Theme::CatppuccinLatte
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Font {
    #[default]
    Default,
    Title,
}

impl Font {
    pub fn family(&self) -> font::Family {
        match self {
            Font::Default => font::Family::SansSerif,
            Font::Title => font::Family::Name("Stigmature"),
        }
    }

    pub fn source(&self) -> &'static [u8] {
        match self {
            Self::Default => &[],
            Self::Title => &include_bytes!("../fonts/stigmature/Stigmature.otf")[..],
        }
    }

    pub fn load(&self) -> Task<()> {
        let font = *self;

        iced::font::load(self.source()).map(move |res| {
            if let Err(err) = res {
                tracing::warn!(?err, ?font, "failed to load font");
            } else {
                tracing::info!("fonts loaded");
            }
        })
    }

    pub fn load_all() -> Task<()> {
        Self::Title.load()
    }
}

impl From<Font> for iced::Font {
    fn from(value: Font) -> Self {
        match value {
            Font::Default => iced::Font::DEFAULT,
            Font::Title => iced::Font {
                family: value.family(),
                ..Default::default()
            },
        }
    }
}
