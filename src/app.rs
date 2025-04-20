use {
    crate::{
        input,
        keyboard::{self, Key, KeyPos},
        util,
        verovio,
    },
    derive_more::{Display, From},
    gloo_storage::Storage as _,
    iced::{Color, Element, Length, Subscription, Task, Theme, font, widget},
    midly::MidiMessage,
    serde::{Deserialize, Serialize},
    tap::TapFallible as _,
};

mod game_active;
mod game_finished;
mod loading;
mod main_menu;

const TITLE: &str = "Clef Rush";
const EXPLAIN_UI: bool = false;

#[derive(Default, Display, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OctaveRange {
    #[default]
    #[display("None")]
    None,

    #[display("{}", _0)]
    Fixed(u8),

    #[display("All")]
    All,
}

impl OctaveRange {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Clef {
    Treble,
    Bass,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub input_device: input::Device,
    pub treble: ClefConfig,
    pub bass: ClefConfig,
}

impl Config {
    const STORAGE_KEY: &str = "global-config";

    pub fn load() -> Self {
        gloo_storage::LocalStorage::get(Self::STORAGE_KEY)
            .tap_err(|err| {
                tracing::info!(?err, "failed to load global config");
            })
            .unwrap_or_default()
    }

    pub fn store(&self) {
        let _ = gloo_storage::LocalStorage::set(Self::STORAGE_KEY, self).tap_err(|err| {
            tracing::info!(?err, "failed to store global config");
        });
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            input_device: input::Device::Virtual,
            treble: ClefConfig {
                clef: Clef::Treble,
                range: OctaveRange::Fixed(2),
                sharp_keys: false,
            },
            bass: ClefConfig {
                clef: Clef::Bass,
                range: OctaveRange::Fixed(2),
                sharp_keys: false,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClefConfig {
    pub clef: Clef,
    pub range: OctaveRange,
    pub sharp_keys: bool,
}

impl ClefConfig {
    pub fn to_key_range(&self) -> Option<Vec<Key>> {
        let (start, end) = match (self.clef, self.range) {
            (Clef::Treble, OctaveRange::Fixed(num)) if num <= 3 => {
                (KeyPos::C.oct(4), KeyPos::B.oct(3 + num))
            }

            (Clef::Treble, OctaveRange::All) => (KeyPos::A.oct(0), KeyPos::B.oct(3)),

            (Clef::Bass, OctaveRange::Fixed(num)) if num <= 3 => {
                (KeyPos::C.oct(4 - num), KeyPos::B.oct(3))
            }

            (Clef::Bass, OctaveRange::All) => (KeyPos::C.oct(4), KeyPos::C.oct(8)),

            _ => return None,
        };

        let range = keyboard::range(&start, &end)
            .filter(|key| self.sharp_keys || key.is_natural())
            .collect();

        Some(range)
    }
}

#[derive(Debug, Clone)]
pub struct GameResults {
    settings: Config,
}

#[derive(Debug, Clone)]
pub enum StateTransition {
    MainMenu,
    GameActive(Config),
    GameFinished(GameResults),
}

#[allow(clippy::large_enum_variant)]
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
    SelectInputPort(input::Device),
    SelectOctaveRange { clef: Clef, range: OctaveRange },
    ToggleSharpKeys { clef: Clef, enabled: bool },
    RefreshDeviceList,
    InputEvent(#[from] MidiMessage),
    InputWorkerReady(input::Connector),
    Ready,
    AdvanceChallenge,
    ToggleVirtualKeyboard,
    ToggleFullscreen,
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

                self.state.init()
            }

            Message::ToggleFullscreen => {
                let _ = util::toggle_fullscreen()
                    .tap_err(|err| tracing::warn!(?err, "failed to toggle fullscreen"));

                Task::none()
            }

            event => match &mut self.state {
                State::Loading(state) => state.update(event),
                State::MainMenu(state) => state.update(event),
                State::GameActive(state) => state.update(event),
                State::GameFinished(state) => state.update(event),
            },
        }
    }

    pub fn view(&self) -> Element<Message> {
        let content = match &self.state {
            State::Loading(state) => state.view(self),
            State::MainMenu(state) => state.view(self),
            State::GameActive(state) => state.view(self),
            State::GameFinished(state) => state.view(self),
        };

        let res: Element<_> = widget::column![content]
            .padding(20)
            .height(Length::Fill)
            .width(Length::Fill)
            .into();

        if EXPLAIN_UI {
            res.explain(Color::BLACK)
        } else {
            res
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
            Self::Title => &include_bytes!("../resources/fonts/stigmature/Stigmature.otf")[..],
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
