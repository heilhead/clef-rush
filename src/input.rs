use {
    crate::app::Message,
    derive_more::Display,
    futures_util::stream,
    iced::{
        Subscription,
        futures::{
            SinkExt,
            Stream,
            StreamExt,
            channel::{
                mpsc::{self, UnboundedSender},
                oneshot,
            },
        },
        keyboard::Key,
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

    #[error("Input worker is not available")]
    WorkerNotAvailable,
}

#[derive(Debug)]
struct ConnectEvent {
    port: PortDescriptor,
    resp: oneshot::Sender<Result<(), Error>>,
}

#[derive(Display, Debug, Clone, PartialEq)]
#[display("{}", name)]
pub struct PortDescriptor {
    id: String,
    name: String,
}

#[derive(Debug, Clone)]
pub struct Connector(UnboundedSender<ConnectEvent>);

impl Connector {
    pub async fn connect(self, port: PortDescriptor) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        let worker_tx = self.0;
        let connect_evt = ConnectEvent { port, resp: tx };

        if worker_tx.unbounded_send(connect_evt).is_err() {
            return Err(Error::WorkerNotAvailable);
        }

        rx.await.map_err(|_| Error::WorkerNotAvailable)??;

        Ok(())
    }
}

struct Connection(Option<MidiInputConnection<()>>);

impl Drop for Connection {
    fn drop(&mut self) {
        self.0.take().unwrap().close();
        tracing::info!("connection closed");
    }
}

pub fn connection_worker() -> impl Stream<Item = Message> {
    let (mut out_tx, out_rx) = mpsc::unbounded();
    let (worker_tx, mut worker_rx) = mpsc::unbounded();

    stream::select(
        out_rx,
        stream::once(async move {
            let _ = out_tx
                .send(Message::InputWorkerReady(Connector(worker_tx)))
                .await;

            let mut _conn = None;

            while let Some(ConnectEvent { port, resp }) = worker_rx.next().await {
                let result = connect(port, out_tx.clone()).map(|conn| {
                    _conn = Some(Connection(Some(conn)));
                });

                let _ = resp.send(result);
            }
        })
        .filter_map(|_| async { None }),
    )
}

pub fn refresh_ports() {
    let _ = midir::MidiInput::new("piano trainer device list")
        .tap_ok(|input| {
            input.ports();
        })
        .tap_err(|err| {
            tracing::warn!(?err, "failed to refresh input ports");
        });
}

pub fn available_ports() -> Result<Vec<PortDescriptor>, Error> {
    let input =
        midir::MidiInput::new("piano trainer device list").map_err(|_| Error::InitFailed)?;

    let ports = input
        .ports()
        .into_iter()
        .map(|port| {
            let name = input
                .port_name(&port)
                .unwrap_or_else(|_| UNKNOWN_PORT_NAME.to_owned());

            PortDescriptor {
                id: port.id(),
                name,
            }
        })
        .collect();

    Ok(ports)
}

fn connect(
    port: PortDescriptor,
    tx: UnboundedSender<Message>,
) -> Result<MidiInputConnection<()>, Error> {
    let input = midir::MidiInput::new("piano-trainer-read-input").map_err(|_| Error::InitFailed)?;

    let port = input
        .find_port_by_id(port.id.clone())
        .ok_or(Error::PortNotAvailable)?;

    input
        .connect(
            &port,
            "piano-trainer-read-input",
            move |stamp, message, _| process_event(stamp, message, &tx),
            (),
        )
        .map_err(|err| Error::PortConnectionFailed(err.to_string()))
}

fn process_event(stamp: u64, message: &[u8], out_tx: &UnboundedSender<Message>) {
    tracing::trace!("{}: {:?} (len = {})", stamp, message, message.len());

    let Ok(event) = LiveEvent::parse(message).tap_err(|err| {
        tracing::warn!(?err, "failed to parse midi message");
    }) else {
        return;
    };

    let event = match event {
        LiveEvent::Midi { message, .. } => match message {
            // Some keyboards send `NoteOn` event with vel 0 instead of `NoteOff`.
            MidiMessage::NoteOn { key, vel } if vel == 0 => Some(MidiMessage::NoteOff { key, vel }),
            MidiMessage::NoteOff { .. } | MidiMessage::NoteOn { .. } => Some(message),
            _ => None,
        },
        _ => None,
    };

    if let Some(event) = event {
        let _ = out_tx.unbounded_send(Message::InputEvent(event));
    }
}

pub mod mock {
    use super::*;

    pub fn subscription() -> Subscription<Message> {
        Subscription::batch([
            iced::keyboard::on_key_press(|key, _| {
                translate_key_note(key)
                    .map(|key| MidiMessage::NoteOn { key, vel: 1.into() })
                    .map(Message::InputEvent)
            }),
            iced::keyboard::on_key_release(|key, _| {
                translate_key_note(key)
                    .map(|key| MidiMessage::NoteOff { key, vel: 0.into() })
                    .map(Message::InputEvent)
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
