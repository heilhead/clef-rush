use {
    derive_more::From,
    iced::{Element, Subscription, Task, Theme, widget},
    midly::MidiMessage,
    tap::TapFallible as _,
    wasm_bindgen::prelude::*,
};

mod input;
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
    fn update(&mut self, event: GlobalEvent) -> Task<GlobalEvent> {
        Task::none()
    }

    fn view<'a>(&'a self, app: &'a App) -> Element<'a, GlobalEvent> {
        widget::column![widget::text("Loading...")].into()
    }

    fn subscription<'a>(&'a self, app: &'a App) -> Subscription<GlobalEvent> {
        Subscription::none()
    }
}

struct MainMenuState {}

impl MainMenuState {
    fn new() -> Self {
        Self {}
    }

    fn update(&mut self, event: GlobalEvent) -> Task<GlobalEvent> {
        Task::none()
    }

    fn view<'a>(&'a self, app: &'a App) -> Element<'a, GlobalEvent> {
        let input = app.input();

        let device_selector = widget::pick_list(input.ports(), input.port(), |port| {
            input::Event::SelectInputPort(port).into()
        })
        .placeholder("Select a device...");

        widget::column![
            widget::button("Refresh Device List").on_press(input::Event::RefreshDeviceList.into()),
            device_selector,
            widget::button("Play").on_press_maybe(input.port().map(|_| {
                GlobalEvent::StateTransition(StateTransitionEvent::GameActive(GameSettings {}))
            }))
        ]
        .into()
    }

    fn subscription<'a>(&'a self, app: &'a App) -> Subscription<GlobalEvent> {
        Subscription::none()
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

    fn update(&mut self, event: GlobalEvent) -> Task<GlobalEvent> {
        match event {
            GlobalEvent::InputEvent(msg) => match msg {
                MidiMessage::NoteOn { key, vel } => {
                    tracing::info!(?key, ?vel, "midi message: note on");
                }

                MidiMessage::NoteOff { key, vel } => {
                    tracing::info!(?key, ?vel, "midi message: note off");
                }

                _ => {}
            },

            _ => {}
        }

        Task::none()
    }

    fn view<'a>(&'a self, app: &'a App) -> Element<'a, GlobalEvent> {
        widget::column![
            widget::button("Finish").on_press(GlobalEvent::StateTransition(
                StateTransitionEvent::GameFinished(GameResults {
                    settings: self.settings.clone()
                })
            ))
        ]
        .into()
    }

    fn subscription<'a>(&'a self, app: &'a App) -> Subscription<GlobalEvent> {
        Subscription::run(input::connection_worker)
        // input::mock::subscription()
    }
}

struct GameFinishedState {
    results: GameResults,
}

impl GameFinishedState {
    fn new(results: GameResults) -> Self {
        Self { results }
    }

    fn update(&mut self, message: GlobalEvent) -> Task<GlobalEvent> {
        Task::none()
    }

    fn view<'a>(&'a self, app: &'a App) -> Element<'a, GlobalEvent> {
        widget::column![
            widget::button("Play Again").on_press(GlobalEvent::StateTransition(
                StateTransitionEvent::GameActive(self.results.settings.clone())
            )),
            widget::button("Main Menu")
                .on_press(GlobalEvent::StateTransition(StateTransitionEvent::MainMenu))
        ]
        .into()
    }

    fn subscription<'a>(&'a self, app: &'a App) -> Subscription<GlobalEvent> {
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
enum StateTransitionEvent {
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
    input: input::Manager,
}

#[derive(From, Debug, Clone)]
enum GlobalEvent {
    StateTransition(StateTransitionEvent),
    InputManager(#[from] input::Event),
    InputEvent(#[from] MidiMessage),
    InputError(#[from] input::Error),
}

impl App {
    fn init() -> (Self, Task<GlobalEvent>) {
        (
            Self {
                state: AppState::Loading(Default::default()),
                input: input::Manager::new(),
            },
            Task::future(verovio::initialize())
                .map(|_| GlobalEvent::StateTransition(StateTransitionEvent::MainMenu)),
        )
    }

    fn update(&mut self, event: GlobalEvent) -> Task<GlobalEvent> {
        match event {
            GlobalEvent::StateTransition(new_state) => match new_state {
                StateTransitionEvent::MainMenu => {
                    self.state = AppState::MainMenu(MainMenuState::new());
                }

                StateTransitionEvent::GameActive(settings) => {
                    self.state = AppState::GameActive(GameActiveState::new(settings));
                }

                StateTransitionEvent::GameFinished(results) => {
                    self.state = AppState::GameFinished(GameFinishedState::new(results));
                }
            },

            GlobalEvent::InputManager(event) => {
                self.input.update(event);
            }

            event => {
                return match &mut self.state {
                    AppState::Loading(state) => state.update(event),
                    AppState::MainMenu(state) => state.update(event),
                    AppState::GameActive(state) => state.update(event),
                    AppState::GameFinished(state) => state.update(event),
                };
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<GlobalEvent> {
        // TODO: Header/footer, global layout.

        match &self.state {
            AppState::Loading(state) => state.view(self),
            AppState::MainMenu(state) => state.view(self),
            AppState::GameActive(state) => state.view(self),
            AppState::GameFinished(state) => state.view(self),
        }
    }

    fn subscription(&self) -> Subscription<GlobalEvent> {
        match &self.state {
            AppState::Loading(state) => state.subscription(self),
            AppState::MainMenu(state) => state.subscription(self),
            AppState::GameActive(state) => state.subscription(self),
            AppState::GameFinished(state) => state.subscription(self),
        }
    }

    fn theme(&self) -> Theme {
        Theme::Light
    }

    pub fn input(&self) -> &input::Manager {
        &self.input
    }
}
