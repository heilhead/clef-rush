use {
    super::{App, Font, GameSettings, Message, StateTransition},
    crate::input::{self, PortDescriptor},
    iced::{Element, Subscription, Task, widget},
};

pub struct State {
    input_ports: Vec<PortDescriptor>,
    input_port: Option<PortDescriptor>,
}

impl State {
    pub fn new() -> Self {
        Self {
            input_ports: input::port_list(),
            input_port: None,
        }
    }

    pub fn init(&mut self) -> Task<Message> {
        Task::none()
    }

    pub fn update(&mut self, event: Message) -> Task<Message> {
        match event {
            Message::RefreshDeviceList => {
                self.input_ports = input::port_list();
            }

            Message::SelectInputPort(port) => {
                self.input_port = Some(port);
            }

            _ => {}
        }

        Task::none()
    }

    pub fn view<'a>(&'a self, _: &'a App) -> Element<'a, Message> {
        let device_selector =
            widget::pick_list(&self.input_ports[..], self.input_port.clone(), |port| {
                Message::SelectInputPort(port)
            })
            .placeholder("Select a device...");

        let btn_play = widget::button("Start");

        let btn_play = if super::USE_MOCK_INPUT {
            btn_play.on_press(Message::StateTransition(StateTransition::GameActive(
                GameSettings {
                    input_port: Default::default(),
                },
            )))
        } else {
            btn_play.on_press_maybe(self.input_port.as_ref().map(|port| {
                Message::StateTransition(StateTransition::GameActive(GameSettings {
                    input_port: port.clone(),
                }))
            }))
        };

        widget::column![
            widget::button("Refresh Device List").on_press(Message::RefreshDeviceList),
            device_selector,
            btn_play
        ]
        .into()
    }

    pub fn subscription<'a>(&'a self, _: &'a App) -> Subscription<Message> {
        Subscription::none()
    }
}
