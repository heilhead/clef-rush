use {
    super::{App, GameResults, GameSettings, Message},
    crate::{
        app::StateTransition,
        input::{self, Connector},
        keyboard::{self, Key, KeyPos},
    },
    iced::{Element, Subscription, Task, widget},
    midly::MidiMessage,
};

mod sheet;

pub struct State {
    settings: GameSettings,
    initialized: bool,
    input: Option<Connector>,
    range_treble: Vec<Key>,
    range_bass: Vec<Key>,
    // svg: Option<widget::svg::Handle>,
}

impl State {
    pub fn new(settings: GameSettings) -> Self {
        Self {
            settings,
            initialized: false,
            input: None,
            range_treble: keyboard::range(&KeyPos::C.oct(4), &KeyPos::B.oct(5)).collect(),
            range_bass: keyboard::range(&KeyPos::C.oct(2), &KeyPos::B.oct(3)).collect(),
        }
    }

    pub fn init(&mut self) -> Task<Message> {
        if super::USE_MOCK_INPUT {
            Task::done(Message::Ready)
        } else {
            Task::none()
        }
    }

    pub fn update(&mut self, event: Message) -> Task<Message> {
        match event {
            Message::InputWorkerReady(connector) => {
                self.input = Some(connector.clone());
                let port = self.settings.input_port.clone();

                return Task::future(async move {
                    let result = connector.connect(port).await;

                    match result {
                        Ok(_) => Message::Ready,

                        Err(err) => {
                            tracing::warn!(?err, "failed to connect input port");
                            StateTransition::MainMenu.into()
                        }
                    }
                });
            }

            Message::Ready => {
                tracing::info!("port connected");
                self.initialized = true;
                return self.advance();
            }

            Message::InputEvent(msg) => match msg {
                MidiMessage::NoteOn { key, vel } => {
                    if let Ok(parsed) = keyboard::Key::try_from_midi(key) {
                        tracing::info!(?key, ?vel, %parsed, "midi message: note on");
                    };
                }

                MidiMessage::NoteOff { key, vel } => {
                    if let Ok(parsed) = keyboard::Key::try_from_midi(key) {
                        tracing::info!(?key, ?vel, %parsed, "midi message: note off");
                    };
                }

                _ => {}
            },

            _ => {}
        }

        Task::none()
    }

    pub fn view<'a>(&'a self, _: &'a App) -> Element<'a, Message> {
        if self.initialized {
            widget::column![widget::button("Finish").on_press(Message::StateTransition(
                StateTransition::GameFinished(GameResults {
                    settings: self.settings.clone()
                })
            ))]
            .into()
        } else {
            widget::column![widget::text("connecting input...")].into()
        }
    }

    pub fn subscription<'a>(&'a self, _: &'a App) -> Subscription<Message> {
        if super::USE_MOCK_INPUT {
            input::mock::subscription()
        } else {
            Subscription::run(input::connection_worker)
        }
    }

    fn advance(&mut self) -> Task<Message> {
        Task::none()
    }
}
