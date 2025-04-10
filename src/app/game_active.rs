use {
    super::{App, GameResults, GameSettings, Message},
    crate::{
        app::StateTransition,
        input::{self, Connector},
        keyboard::{self, Key, KeyPos},
    },
    iced::{Element, Subscription, Task, widget},
    midly::MidiMessage,
    rand::seq::IndexedRandom,
    std::{borrow::Cow, collections::HashSet},
};

mod sheet;

pub struct State {
    settings: GameSettings,
    initialized: bool,
    input: Option<Connector>,
    range_treble: Vec<Key>,
    range_bass: Vec<Key>,
    challenge: Option<Challenge>,
}

impl State {
    pub fn new(settings: GameSettings) -> Self {
        Self {
            settings,
            initialized: false,
            input: None,
            range_treble: keyboard::range(&KeyPos::G.oct(3), &KeyPos::C.oct(5))
                .filter(|key| key.pos.is_neutral())
                .collect(),
            range_bass: keyboard::range(&KeyPos::C.oct(2), &KeyPos::B.oct(3))
                .filter(|key| key.pos.is_neutral())
                .collect(),
            challenge: None,
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

            Message::NextChallenge(challenge) => {
                self.challenge = Some(challenge);
            }

            Message::InputEvent(msg) => match msg {
                MidiMessage::NoteOn { key, vel } => {
                    if let Ok(key) = Key::try_from_midi(key) {
                        tracing::info!(?key, ?vel, "midi message: note on");

                        if let Some(challenge) = &mut self.challenge {
                            if challenge.validator.validate(key) {
                                tracing::info!(?key, "correct key");

                                if challenge.validator.finished() {
                                    return self.advance();
                                }
                            } else {
                                tracing::info!(?key, "incorrect key");
                            }
                        }
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
            let hint: Element<'a, Message> = if let Some(challenge) = &self.challenge {
                widget::svg(challenge.hint.clone()).into()
            } else {
                widget::text("loading...").into()
            };

            widget::column![
                hint,
                widget::button("Finish").on_press(Message::StateTransition(
                    StateTransition::GameFinished(GameResults {
                        settings: self.settings.clone()
                    })
                ))
            ]
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
        self.challenge = None;

        let treble_note =
            sheet::Notes::Single(*self.range_treble[..].choose(&mut rand::rng()).unwrap());
        let bass_note =
            sheet::Notes::Single(*self.range_bass[..].choose(&mut rand::rng()).unwrap());

        Task::future(async move {
            let instant = instant::Instant::now();
            let hint = sheet::generate_svg(Some(&treble_note), None).await;
            tracing::info!(elapsed = ?instant.elapsed(), %hint, "generated svg");

            let hint = iced::widget::svg::Handle::from_memory(Cow::Owned(hint.as_bytes().into()));

            Message::NextChallenge(Challenge::new(Some(&treble_note), None, hint))
        })
    }
}

#[derive(Debug, Clone)]
pub struct Challenge {
    validator: Validator,
    hint: widget::svg::Handle,
}

impl Challenge {
    fn new(
        treble: Option<&sheet::Notes>,
        bass: Option<&sheet::Notes>,
        hint: widget::svg::Handle,
    ) -> Self {
        Self {
            validator: Validator::new(treble, bass),
            hint,
        }
    }
}

#[derive(Debug, Clone)]
struct Validator {
    expected: HashSet<Key>,
    validated: HashSet<Key>,
}

impl Validator {
    fn new(treble: Option<&sheet::Notes>, bass: Option<&sheet::Notes>) -> Self {
        let mut expected = HashSet::new();

        if let Some(notes) = treble {
            expected.extend(notes.keys());
        }

        if let Some(notes) = bass {
            expected.extend(notes.keys());
        }

        Self {
            expected,
            validated: HashSet::new(),
        }
    }

    fn validate(&mut self, key: Key) -> bool {
        if self.expected.remove(&key) {
            self.validated.insert(key);
            true
        } else {
            self.validated.contains(&key)
        }
    }

    fn finished(&self) -> bool {
        self.expected.is_empty()
    }
}
