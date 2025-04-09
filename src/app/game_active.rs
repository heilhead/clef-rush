use {
    super::{App, GameResults, GameSettings, Message},
    crate::{
        app::StateTransition,
        input::{self, Connector},
    },
    iced::{Element, Subscription, Task, widget},
    midly::MidiMessage,
};

pub struct State {
    settings: GameSettings,
    initialized: bool,
    input: Option<Connector>,
    // svg: Option<widget::svg::Handle>,
}

impl State {
    pub fn new(settings: GameSettings) -> Self {
        Self {
            settings,
            initialized: false,
            input: None,
        }
    }

    pub fn init(&mut self) -> Task<Message> {
        Task::none()
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
            }

            Message::InputEvent(msg) => match msg {
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
        Subscription::run(input::connection_worker)
        // input::mock::subscription()
    }
}
