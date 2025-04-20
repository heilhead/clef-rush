use {
    super::{App, Config, Font, Message},
    crate::{
        app::StateTransition,
        input::{self, Connector},
        keyboard::{self, Key, KeyPos, Keyboard},
        piano::{self, Piano},
        util,
    },
    gloo_storage::Storage as _,
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
    serde::{Deserialize, Serialize},
    sheet::{Note, Sheet},
    smallvec::SmallVec,
    std::{borrow::Cow, collections::HashSet},
    tap::TapFallible as _,
};

mod sheet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    pub virtual_keyboard: bool,
}

impl Default for LocalConfig {
    fn default() -> Self {
        Self {
            virtual_keyboard: true,
        }
    }
}

impl LocalConfig {
    const STORAGE_KEY: &str = "ingame-config";

    pub fn load() -> Self {
        gloo_storage::LocalStorage::get(Self::STORAGE_KEY)
            .tap_err(|err| {
                tracing::info!(?err, "failed to load ingame local config");
            })
            .unwrap_or_default()
    }

    pub fn store(&self) {
        let _ = gloo_storage::LocalStorage::set(Self::STORAGE_KEY, self).tap_err(|err| {
            tracing::info!(?err, "failed to store ingame local config");
        });
    }
}

pub struct State {
    config: Config,
    local_config: LocalConfig,
    initialized: bool,
    input: Option<Connector>,
    range_treble: Option<Vec<Key>>,
    range_bass: Option<Vec<Key>>,
    curr_challenge: Option<Challenge>,
    prev_challenge: Option<Challenge>,
    hint: Option<widget::svg::Handle>,
    piano: Piano,
}

impl State {
    pub fn new(config: Config) -> Self {
        let range_treble = config.treble.to_key_range();
        let range_bass = config.bass.to_key_range();

        Self {
            config,
            local_config: LocalConfig::load(),
            initialized: false,
            input: None,
            range_treble,
            range_bass,
            curr_challenge: None,
            prev_challenge: None,
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
                let device = self.config.input_device.clone();

                return match device {
                    input::Device::Virtual => Task::done(Message::Ready),

                    input::Device::Midi(port) => Task::future(async move {
                        match connector.connect(port).await {
                            Ok(_) => Message::Ready,

                            Err(err) => {
                                tracing::warn!(?err, "failed to connect input port");
                                // TODO: Reset input device selection.
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

            Message::ToggleVirtualKeyboard => {
                self.local_config.virtual_keyboard = !self.local_config.virtual_keyboard;
                self.local_config.store();
            }

            Message::InputEvent(msg) => match msg {
                MidiMessage::NoteOn { key, vel } => {
                    if let Ok(key) = Key::try_from_midi(key) {
                        tracing::info!(?key, ?vel, "midi message: note on");

                        if let Some(challenge) = &mut self.curr_challenge {
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

                                    self.prev_challenge = self.curr_challenge.take();
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

                        if let Some(challenge) = &mut self.curr_challenge {
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
            widget::button("Toggle Fullscreen").on_press(Message::ToggleFullscreen),
            widget::button("Toggle Keyboard").on_press(Message::ToggleVirtualKeyboard),
            widget::button("Skip").on_press(Message::AdvanceChallenge),
            widget::button("Main Menu")
                .on_press(Message::StateTransition(StateTransition::MainMenu))
        ]
        .spacing(20)
        .align_y(alignment::Vertical::Center)
        .width(Length::Fill);

        let content = if self.initialized {
            let hint: Element<_> = if let Some(hint) = &self.hint {
                widget::svg(hint.clone())
                    .height(Length::Fixed(500.))
                    .width(Length::Fill)
                    .into()
            } else {
                widget::text("Loading...").into()
            };

            widget::column![widget::vertical_space(), hint, widget::vertical_space()]
                .push_maybe(self.local_config.virtual_keyboard.then(|| {
                    Container::new(self.piano.view())
                        .height(Length::Fixed(150.))
                        .width(Length::Fill)
                }))
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
        let kbd = Keyboard::standard_88_key();
        match (&self.range_treble, &self.range_bass) {
            (None, Some(_)) => kbd.last(),
            (Some(_), None) => kbd.first(),
            _ => KeyPos::C.oct(4),
        }
    }

    fn advance(&mut self) -> Task<Message> {
        let choose_note = |range: &Vec<Key>| {
            loop {
                let key = range[..].choose(&mut rand::rng()).unwrap();

                let is_repeated = self
                    .prev_challenge
                    .as_ref()
                    .map(|challenge| challenge.validator.required(*key))
                    .unwrap_or(false);

                if !is_repeated {
                    break *key;
                }
            }
        };

        let treble = self.range_treble.as_ref().map(choose_note).map(Note::from);
        let bass = self.range_bass.as_ref().map(choose_note).map(Note::from);
        let notes = treble.into_iter().chain(bass).collect::<SmallVec<[_; 2]>>();

        self.curr_challenge = Some(Challenge::new(&notes, self.clef_split()));
        self.update_hint()
    }

    fn update_hint(&self) -> Task<Message> {
        let Some(challenge) = &self.curr_challenge else {
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
        let mut treble_ntoes = false;
        let mut bass_notes = false;

        for note in notes {
            if note.key >= clef_split {
                treble_ntoes = true;
            } else {
                bass_notes = true;
            }
        }

        let mode = match (treble_ntoes, bass_notes) {
            (true, false) => sheet::Mode::Treble,
            (false, true) => sheet::Mode::Bass,
            _ => sheet::Mode::Combined,
        };

        Self {
            validator: Validator::new(notes),
            sheet: Sheet::new(mode, notes, clef_split),
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
