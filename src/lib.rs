use {
    iced::{
        Center,
        Length,
        Subscription,
        Task,
        Theme,
        time,
        widget::{self, Column, svg},
    },
    midly::{MidiMessage, live::LiveEvent},
    std::{borrow::Cow, time::Duration},
    tap::TapFallible as _,
    wasm_bindgen::prelude::*,
};

mod util;
mod verovio;

#[wasm_bindgen(start)]
pub async fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    tracing_wasm::set_as_global_default();

    let mut midi_in = midir::MidiInput::new("midir reading input").unwrap();
    midi_in.ignore(midir::Ignore::None);

    let mut ports;

    let in_port = loop {
        // Get an input port
        ports = midi_in.ports();

        match &ports[..] {
            [] => {
                tracing::info!("No ports available yet, will try again");
                util::sleep(2000).await;
                continue;
            }
            [port] => {
                tracing::info!(
                    "Choosing the only available input port: {}",
                    midi_in.port_name(port).unwrap()
                );
                break port;
            }
            _ => {
                break &ports[0];
            }
        };
    };

    tracing::info!("Opening connection");
    let in_port_name = midi_in.port_name(in_port).unwrap();

    // _conn_in needs to be a named parameter, because it needs to be kept alive
    // until the end of the scope
    let _conn_in = midi_in
        .connect(
            in_port,
            "midir-read-input",
            move |stamp, message, _| {
                tracing::info!("{}: {:?} (len = {})", stamp, message, message.len());

                // TODO: Some keyboards send NoteOn event with vel 0 instead of NoteOff.
                let event = LiveEvent::parse(message).unwrap();
                match event {
                    LiveEvent::Midi { channel, message } => match message {
                        MidiMessage::NoteOn { key, vel } => {
                            tracing::info!(?key, ?vel, ?channel, "hit note");
                        }
                        _ => {}
                    },
                    _ => {}
                }
            },
            (),
        )
        .unwrap();

    tracing::info!("Connection open, reading input from '{}'", in_port_name);
    Box::leak(Box::new(_conn_in));

    iced::application("Piano Trainer", Trainer::update, Trainer::view)
        .subscription(Trainer::subscription)
        .theme(Trainer::theme)
        .run_with(Trainer::new)
        .tap_err(|err| tracing::error!(?err, "iced app failed"))
        .ok();
}

struct Trainer {
    initialized: bool,
    svg: Option<svg::Handle>,
}

#[derive(Debug, Clone)]
enum Message {
    Initialized,
    Tick,
    UpdateSvg(svg::Handle),
}

impl Trainer {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                initialized: false,
                svg: None,
            },
            Task::future(verovio::initialize()).map(|_| Message::Initialized),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Initialized => self.initialized = true,
            Message::Tick => {
                return Task::future(verovio::update_svg()).map(|data| {
                    Message::UpdateSvg(svg::Handle::from_memory(Cow::Owned(data.as_bytes().into())))
                });
            }
            Message::UpdateSvg(handle) => self.svg = Some(handle),
        }

        Task::none()
    }

    fn view(&self) -> Column<Message> {
        if !self.initialized {
            return widget::column![];
        }

        let col = if let Some(handle) = &self.svg {
            widget::column![svg(handle.clone()).width(Length::Fill).height(Length::Fill)]
        } else {
            widget::column![
                widget::text("hello world"),
                // button("Increment").on_press(Message::Increment),
                // button("Decrement").on_press(Message::Decrement),
            ]
        };

        col.padding(20).align_x(Center)
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(Duration::from_secs(10)).map(|_| Message::Tick)
    }

    fn theme(&self) -> Theme {
        Theme::Light
    }
}
