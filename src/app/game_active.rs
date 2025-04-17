use {
    super::{App, Font, GameConfig, Message},
    crate::{
        app::StateTransition,
        input::{self, Connector},
        keyboard::{self, Key, KeyPos},
        piano::{self, Piano},
        util,
    },
    iced::{
        Element,
        Length,
        Subscription,
        Task,
        alignment,
        widget::{self, Container},
    },
    midly::MidiMessage,
    rand::seq::IndexedRandom,
    sheet::{Note, Sheet},
    std::{borrow::Cow, collections::HashSet},
};

mod sheet;

pub struct State {
    settings: GameConfig,
    initialized: bool,
    input: Option<Connector>,
    range_treble: Vec<Key>,
    range_bass: Vec<Key>,
    challenge: Option<Challenge>,
    hint: Option<widget::svg::Handle>,
    piano: Piano,
}

impl State {
    pub fn new(settings: GameConfig) -> Self {
        let mut range_treble = keyboard::range(&KeyPos::B.oct(3), &KeyPos::C.oct(5))
            .filter(|key| key.pos.is_natural())
            .collect::<Vec<_>>();
        range_treble.sort();

        let mut range_bass = keyboard::range(&KeyPos::C.oct(2), &KeyPos::A.oct(3))
            .filter(|key| key.pos.is_natural())
            .collect::<Vec<_>>();
        range_bass.sort();

        Self {
            settings,
            initialized: false,
            input: None,
            range_treble,
            range_bass,
            challenge: None,
            hint: None,
            piano: Piano::new(keyboard::Keyboard::standard_88_key()),
        }
    }

    pub fn init(&mut self) -> Task<Message> {
        Task::none()
    }

    pub fn update(&mut self, event: Message) -> Task<Message> {
        match event {
            Message::InputWorkerReady(connector) => {
                self.input = Some(connector.clone());
                let device = self.settings.input_device.clone();

                return match device {
                    input::Device::Virtual => Task::done(Message::Ready),

                    input::Device::Midi(port) => Task::future(async move {
                        match connector.connect(port).await {
                            Ok(_) => Message::Ready,

                            Err(err) => {
                                tracing::warn!(?err, "failed to connect input port");
                                StateTransition::MainMenu.into()
                            }
                        }
                    }),
                };
            }

            Message::Ready => {
                tracing::info!("port connected");
                self.initialized = true;
                return self.advance();
            }

            Message::UpdateChallengeHint(hint) => {
                self.hint = Some(hint);
            }

            Message::AdvanceChallenge => {
                return self.advance();
            }

            Message::InputEvent(msg) => match msg {
                MidiMessage::NoteOn { key, vel } => {
                    if let Ok(key) = Key::try_from_midi(key) {
                        tracing::info!(?key, ?vel, "midi message: note on");

                        if let Some(challenge) = &mut self.challenge {
                            self.piano.set_key_state(key, piano::KeyState::Pressed);

                            if challenge.validator.validate(key) {
                                tracing::info!(?key, "correct key");

                                challenge.sheet.set_note_style(key, sheet::Style::Correct);

                                if challenge.validator.finished() {
                                    let tasks = Task::batch([
                                        self.update_hint(),
                                        Task::future(async {
                                            util::sleep(500).await;
                                            Message::AdvanceChallenge
                                        }),
                                    ]);

                                    self.challenge = None;
                                    return tasks;
                                }
                            } else {
                                challenge.sheet.add_note(key, sheet::Style::Incorrect);

                                tracing::info!(?key, "incorrect key");
                            }

                            return self.update_hint();
                        }
                    };
                }

                MidiMessage::NoteOff { key, vel } => {
                    if let Ok(key) = Key::try_from_midi(key) {
                        tracing::info!(?key, ?vel, "midi message: note off");

                        self.piano.set_key_state(key, piano::KeyState::Released);

                        if let Some(challenge) = &mut self.challenge {
                            if !challenge.validator.required(key) {
                                challenge.sheet.remove_note(key);
                                return self.update_hint();
                            }
                        }
                    }
                }

                _ => {}
            },

            _ => {}
        }

        Task::none()
    }

    pub fn view<'a>(&'a self, _: &'a App) -> Element<'a, Message> {
        let header = widget::row![
            widget::text(super::TITLE).size(36).font(Font::Title),
            widget::horizontal_space(),
            widget::button("Next").on_press(Message::AdvanceChallenge),
            widget::button("Main Menu")
                .on_press(Message::StateTransition(StateTransition::MainMenu))
        ]
        .spacing(20)
        .align_y(alignment::Vertical::Center)
        .width(Length::Fill);

        let content = if self.initialized {
            let hint: Element<'a, Message> = if let Some(hint) = &self.hint {
                widget::svg(hint.clone())
                    .height(Length::Fixed(500.))
                    .width(Length::Fill)
                    .into()
            } else {
                widget::text("Loading...").into()
            };

            let piano = Container::new(self.piano.view())
                .height(Length::Fixed(150.))
                .width(Length::Fill);

            widget::column![
                widget::vertical_space(),
                hint,
                widget::vertical_space(),
                piano
            ]
            .width(Length::Fill)
        } else {
            widget::column![
                widget::vertical_space(),
                widget::text("Connecting input..."),
                widget::vertical_space(),
            ]
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(alignment::Horizontal::Center)
        };

        widget::column![header, content]
            .spacing(10)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    }

    pub fn subscription<'a>(&'a self, _: &'a App) -> Subscription<Message> {
        Subscription::run(input::connection_worker)
    }

    fn clef_split(&self) -> Key {
        // TODO: Support separate treble/bass.
        self.range_treble
            .first()
            .cloned()
            .unwrap_or(KeyPos::C.oct(0))
    }

    fn advance(&mut self) -> Task<Message> {
        self.challenge = None;

        let treble_key = *self.range_treble[..].choose(&mut rand::rng()).unwrap();
        let bass_key = *self.range_bass[..].choose(&mut rand::rng()).unwrap();
        let notes = [treble_key.into(), bass_key.into()];

        self.challenge = Some(Challenge::new(&notes, self.clef_split()));
        self.update_hint()
    }

    fn update_hint(&self) -> Task<Message> {
        let Some(challenge) = &self.challenge else {
            return Task::none();
        };

        let hint_fut = challenge.sheet.render_hint_svg();

        Task::future(async move {
            let instant = instant::Instant::now();
            let hint = hint_fut.await;
            tracing::info!(elapsed = ?instant.elapsed(), "generated svg");

            Message::UpdateChallengeHint(widget::svg::Handle::from_memory(Cow::Owned(
                hint.as_bytes().into(),
            )))
        })
    }
}

#[derive(Debug, Clone)]
pub struct Challenge {
    validator: Validator,
    sheet: Sheet,
}

impl Challenge {
    fn new(notes: &[Note], clef_split: Key) -> Self {
        let sheet = Sheet::new(false, notes, clef_split);

        Self {
            validator: Validator::new(notes),
            sheet,
        }
    }
}

#[derive(Debug, Clone)]
struct Validator {
    expected: HashSet<Key>,
    validated: HashSet<Key>,
}

impl Validator {
    fn new(notes: &[Note]) -> Self {
        let mut expected = HashSet::new();
        expected.extend(notes.iter().map(|note| note.key));

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

    fn required(&self, key: Key) -> bool {
        self.expected.contains(&key) || self.validated.contains(&key)
    }
}
