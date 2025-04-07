use {
    iced::{Element, Subscription, Task, Theme, widget},
    midly::{MidiMessage, live::LiveEvent},
    std::sync::{Arc, Mutex},
    tap::{Tap, TapFallible as _},
    wasm_bindgen::prelude::*,
};

mod util;
mod verovio;

#[wasm_bindgen(start)]
pub async fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    tracing_wasm::set_as_global_default();

    // modal, pick_list, loading_spinners, layout, checkbox

    // // _conn_in needs to be a named parameter, because it needs to be kept alive
    // // until the end of the scope
    // let _conn_in = midi_in
    //     .connect(
    //         in_port,
    //         "midir-read-input",
    //         move |stamp, message, _| {
    //             tracing::info!("{}: {:?} (len = {})", stamp, message,
    // message.len());

    //             // TODO: Some keyboards send NoteOn event with vel 0 instead of
    // NoteOff.             let event = LiveEvent::parse(message).unwrap();
    //             match event {
    //                 LiveEvent::Midi { channel, message } => match message {
    //                     MidiMessage::NoteOn { key, vel } => {
    //                         tracing::info!(?key, ?vel, ?channel, "hit note");
    //                     }
    //                     _ => {}
    //                 },
    //                 _ => {}
    //             }
    //         },
    //         (),
    //     )
    //     .unwrap();

    // Box::leak(Box::new(_conn_in));

    iced::application("Piano Trainer", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .run_with(App::init)
        .tap_err(|err| tracing::error!(?err, "iced app failed"))
        .ok();
}

#[derive(Default)]
struct LoadingState {}

impl LoadingState {
    fn update(&mut self, message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        widget::column![widget::text("Loading...")].into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

#[derive(derive_more::Display, Debug, Clone, PartialEq)]
#[display("{}", name)]
struct MidiPortDescriptor {
    name: String,
    index: usize,
}

struct MainMenuState {
    midi_in: Option<midir::MidiInput>,
    midi_port_selected: Option<MidiPortDescriptor>,
    midi_ports: Option<(Vec<midir::MidiInputPort>, Vec<MidiPortDescriptor>)>,
}

impl MainMenuState {
    fn new() -> Self {
        Self {
            midi_in: Self::try_init_midi(),
            midi_port_selected: None,
            midi_ports: None,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::RefreshMidiDeviceList => {
                if self.midi_in.is_none() {
                    self.midi_in = Self::try_init_midi();
                }

                if let Some(midi_in) = &self.midi_in {
                    let ports = midi_in.ports();

                    let descriptors = ports
                        .iter()
                        .enumerate()
                        .map(|(index, port)| MidiPortDescriptor {
                            name: midi_in
                                .port_name(port)
                                .unwrap_or_else(|_| "Unknown".to_owned()),
                            index,
                        })
                        .collect();

                    self.midi_ports = Some((ports, descriptors));
                }
            }

            Message::SelectMidiPort(port) => {
                self.midi_port_selected = Some(port);
            }

            _ => {}
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let device_selector = widget::pick_list(
            self.midi_ports
                .as_ref()
                .map(|(_, descriptors)| &descriptors[..])
                .unwrap_or_default(),
            self.midi_port_selected.clone(),
            Message::SelectMidiPort,
        )
        .placeholder("Select a device...");

        widget::column![
            widget::button("Refresh Device List").on_press(Message::RefreshMidiDeviceList),
            device_selector,
            widget::button("Play").on_press(Message::StateTransition(StateTransition::GameActive(
                GameSettings {}
            )))
        ]
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }

    fn try_init_midi() -> Option<midir::MidiInput> {
        midir::MidiInput::new("midir reading input")
            .tap_ok_mut(|midi_in| {
                midi_in.ignore(midir::Ignore::None);
            })
            .tap_err(|err| {
                tracing::warn!(?err, "failed to init midi");
            })
            .ok()
    }
}

struct GameActiveState {
    settings: GameSettings,
    // svg: Option<widget::svg::Handle>,
}

impl GameActiveState {
    fn new(settings: GameSettings) -> Self {
        Self { settings }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        widget::column![widget::button("Finish").on_press(Message::StateTransition(
            StateTransition::GameFinished(GameResults {
                settings: self.settings.clone()
            })
        ))]
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        // time::every(Duration::from_secs(10)).map(|_| Message::Tick)
        Subscription::none()
    }
}

struct GameFinishedState {
    results: GameResults,
}

impl GameFinishedState {
    fn new(results: GameResults) -> Self {
        Self { results }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<Message> {
        widget::column![
            widget::button("Play Again").on_press(Message::StateTransition(
                StateTransition::GameActive(self.results.settings.clone())
            )),
            widget::button("Main Menu")
                .on_press(Message::StateTransition(StateTransition::MainMenu))
        ]
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::none()
    }
}

#[derive(Debug, Clone)]
struct GameSettings {}

#[derive(Debug, Clone)]
struct GameResults {
    settings: GameSettings,
}

#[derive(Debug, Clone)]
enum StateTransition {
    MainMenu,
    GameActive(GameSettings),
    GameFinished(GameResults),
}

enum AppState {
    Loading(LoadingState),
    MainMenu(MainMenuState),
    GameActive(GameActiveState),
    GameFinished(GameFinishedState),
}

struct App {
    state: AppState,
}

#[derive(Debug, Clone)]
enum Message {
    StateTransition(StateTransition),
    PlayGame,
    FinishGame,
    RefreshMidiDeviceList,
    SelectMidiPort(MidiPortDescriptor),
}

impl App {
    fn init() -> (Self, Task<Message>) {
        (
            Self {
                state: AppState::Loading(Default::default()),
            },
            Task::future(verovio::initialize())
                .map(|_| Message::StateTransition(StateTransition::MainMenu)),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::StateTransition(new_state) => match new_state {
                StateTransition::MainMenu => {
                    self.state = AppState::MainMenu(MainMenuState::new());
                }

                StateTransition::GameActive(settings) => {
                    self.state = AppState::GameActive(GameActiveState::new(settings));
                }

                StateTransition::GameFinished(results) => {
                    self.state = AppState::GameFinished(GameFinishedState::new(results));
                }
            },

            message => {
                return match &mut self.state {
                    AppState::Loading(state) => state.update(message),
                    AppState::MainMenu(state) => state.update(message),
                    AppState::GameActive(state) => state.update(message),
                    AppState::GameFinished(state) => state.update(message),
                };
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        // TODO: Header/footer, global layout.

        match &self.state {
            AppState::Loading(state) => state.view(),
            AppState::MainMenu(state) => state.view(),
            AppState::GameActive(state) => state.view(),
            AppState::GameFinished(state) => state.view(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        match &self.state {
            AppState::Loading(state) => state.subscription(),
            AppState::MainMenu(state) => state.subscription(),
            AppState::GameActive(state) => state.subscription(),
            AppState::GameFinished(state) => state.subscription(),
        }
    }

    fn theme(&self) -> Theme {
        Theme::Light
    }
}
