use {
    super::GlobalEvent,
    derive_more::{Display, From},
    iced::{
        Subscription,
        futures::{
            SinkExt,
            Stream,
            StreamExt,
            channel::mpsc::{self, Sender, UnboundedReceiver, UnboundedSender},
        },
        keyboard::Key,
        stream,
    },
    midir::MidiInputConnection,
    midly::{MidiMessage, live::LiveEvent, num::u7},
    tap::TapFallible as _,
};

const UNKNOWN_PORT_NAME: &str = "Unknown";

#[derive(Debug, thiserror::Error, Clone)]
pub enum Error {
    #[error("Failed to initialize midi input")]
    InitFailed,

    #[error("Specified input port is not available")]
    PortNotAvailable,

    #[error("Input port connection failed: {0}")]
    PortConnectionFailed(String),
}

#[derive(Debug, Clone)]
pub enum Event {
    RefreshDeviceList,
    SelectInputPort(PortDescriptor),
    WorkerReady(UnboundedSender<GlobalEvent>),
    Connect(PortDescriptor),
    ConnectionResult(Result<(), Error>),
    Disconnect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortHandle(usize);

#[derive(Display, Debug, Clone, PartialEq)]
#[display("{}", name)]
pub struct PortDescriptor {
    id: String,
    name: String,
    handle: PortHandle,
}

pub struct Manager {
    input: Option<midir::MidiInput>,
    port: Option<(midir::MidiInputPort, PortDescriptor)>,
    ports: Option<(Vec<midir::MidiInputPort>, Vec<PortDescriptor>)>,
    worker_tx: Option<UnboundedSender<GlobalEvent>>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            input: None,
            port: None,
            ports: None,
            worker_tx: None,
        }
    }

    pub fn update(&mut self, event: Event) {
        match event {
            Event::RefreshDeviceList => {
                if self.input.is_none() {
                    self.input = midir::MidiInput::new("piano trainer input device list")
                        .tap_ok_mut(|midi_in| {
                            midi_in.ignore(midir::Ignore::None);
                        })
                        .tap_err(|err| {
                            tracing::warn!(?err, "failed to init midi");
                        })
                        .ok();
                }

                if let Some(input) = &self.input {
                    let ports = input.ports();

                    let descriptors = ports
                        .iter()
                        .enumerate()
                        .map(|(index, port)| PortDescriptor {
                            id: port.id(),
                            name: input
                                .port_name(port)
                                .unwrap_or_else(|_| UNKNOWN_PORT_NAME.to_owned()),
                            handle: PortHandle(index),
                        })
                        .collect();

                    tracing::info!(?descriptors, "ports refreshed");

                    let ports = Some((ports, descriptors));

                    if self.ports != ports {
                        self.port = None;
                    }

                    self.ports = ports;
                }
            }

            Event::SelectInputPort(port) => {
                self.set_port(port)
                    .tap_err(|err| {
                        tracing::warn!(?err, "failed to select input port");
                    })
                    .ok();
            }

            Event::WorkerReady(tx) => {
                tx.unbounded_send(Event::Connect(self.port.as_ref().unwrap().1.clone()).into())
                    .unwrap();
                self.worker_tx = Some(tx);
            }

            Event::ConnectionResult(res) => {
                tracing::info!(?res, "midi connection result");
            }

            _ => {}
        }
    }

    pub fn ports(&self) -> &[PortDescriptor] {
        self.ports
            .as_ref()
            .map(|(_, ports)| &ports[..])
            .unwrap_or_default()
    }

    pub fn port(&self) -> Option<PortDescriptor> {
        self.port.as_ref().map(|(_, desc)| desc.clone())
    }

    fn set_port(&mut self, desc: PortDescriptor) -> Result<(), Error> {
        let ports = self
            .ports
            .as_ref()
            .map(|(ports, _)| &ports[..])
            .unwrap_or_default();

        let idx = desc.handle.0;

        if idx < ports.len() {
            self.port = Some((ports[idx].clone(), desc));
            Ok(())
        } else {
            Err(Error::PortNotAvailable)
        }
    }
}

#[derive(From)]
pub struct ConnectionHandle(#[from] MidiInputConnection<()>);

pub fn connection_worker() -> impl Stream<Item = GlobalEvent> {
    let (mut out_tx, out_rx) = mpsc::unbounded();
    let (worker_tx, mut worker_rx) = mpsc::unbounded::<GlobalEvent>();

    futures_util::stream::select(
        out_rx,
        futures_util::stream::once(async move {
            out_tx
                .send(GlobalEvent::InputManager(Event::WorkerReady(worker_tx)))
                .await
                .unwrap();

            let mut _handle = None;

            loop {
                match worker_rx.select_next_some().await {
                    GlobalEvent::InputManager(evt) => match evt {
                        Event::Connect(port) => match connect(port, out_tx.clone()) {
                            Ok(handle) => {
                                _handle = Some(handle);

                                out_tx
                                    .send(Event::ConnectionResult(Ok(())).into())
                                    .await
                                    .unwrap();
                            }

                            Err(err) => {
                                out_tx
                                    .send(Event::ConnectionResult(Err(err)).into())
                                    .await
                                    .unwrap();
                            }
                        },

                        Event::Disconnect => {
                            _handle = None;
                        }

                        _ => {}
                    },

                    _ => {}
                }
            }
        })
        .filter_map(|_| async { None }),
    )
}

fn connect(
    port: PortDescriptor,
    tx: UnboundedSender<GlobalEvent>,
) -> Result<ConnectionHandle, Error> {
    let mut midi_in =
        midir::MidiInput::new("piano trainer input").map_err(|_| Error::InitFailed)?;
    midi_in.ignore(midir::Ignore::None);

    let port = midi_in
        .find_port_by_id(port.id.clone())
        .ok_or(Error::PortNotAvailable)?;

    let handle = midi_in
        .connect(
            &port,
            "piano-trainer-read-input",
            move |stamp, message, _| {
                tracing::info!("{}: {:?} (len = {})", stamp, message, message.len());

                let Ok(event) = LiveEvent::parse(message).tap_err(|err| {
                    tracing::warn!(?err, "failed to parse midi message");
                }) else {
                    return;
                };

                let event = match event {
                    LiveEvent::Midi { message, .. } => match message {
                        // Some keyboards send `NoteOn` event with vel 0 instead of `NoteOff`.
                        MidiMessage::NoteOn { key, vel } if vel == 0 => {
                            Some(MidiMessage::NoteOff { key, vel })
                        }

                        MidiMessage::NoteOff { .. } | MidiMessage::NoteOn { .. } => Some(message),

                        _ => None,
                    },

                    _ => None,
                };

                if let Some(event) = event {
                    tx.unbounded_send(super::GlobalEvent::InputEvent(event))
                        .tap_err(|_| {
                            tracing::warn!("midi input event channel closed");
                        })
                        .ok();
                }
            },
            (),
        )
        .map_err(|err| Error::PortConnectionFailed(err.to_string()))?;

    Ok(handle.into())
}

pub mod mock {
    use super::*;

    pub fn subscription() -> Subscription<GlobalEvent> {
        Subscription::batch([
            iced::keyboard::on_key_press(|key, _| {
                translate_key_note(key)
                    .map(|key| MidiMessage::NoteOn { key, vel: 1.into() })
                    .map(GlobalEvent::InputEvent)
            }),
            iced::keyboard::on_key_release(|key, _| {
                translate_key_note(key)
                    .map(|key| MidiMessage::NoteOff { key, vel: 0.into() })
                    .map(GlobalEvent::InputEvent)
            }),
        ])
    }

    fn translate_key_note(key: Key) -> Option<u7> {
        const BASE_NOTE_OFFSET: u8 = 75;

        match key {
            Key::Character(code) => match code.chars().next() {
                Some(code @ 'a'..='z') => Some((code as u8 - 'a' as u8 + BASE_NOTE_OFFSET).into()),
                _ => None,
            },
            _ => None,
        }
    }
}
