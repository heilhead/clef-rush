use {
    super::{App, Config, Message, OctaveRange, StateTransition},
    crate::{
        app::{self, Clef, Font},
        input,
    },
    iced::{Element, Length, Subscription, Task, alignment, widget},
};

pub struct State {
    input_devices: Vec<input::Device>,
    config: Config,
}

impl State {
    pub fn new() -> Self {
        let mut state = Self {
            input_devices: Vec::new(),
            config: Config::load(),
        };
        state.update_input_devices();
        state
    }

    pub fn init(&mut self) -> Task<Message> {
        Task::none()
    }

    pub fn update(&mut self, event: Message) -> Task<Message> {
        match event {
            Message::RefreshDeviceList => {
                self.update_input_devices();
            }

            Message::SelectInputPort(port) => {
                self.config.input_device = port;
                self.config.store();
            }

            Message::SelectOctaveRange { clef, range } => {
                match clef {
                    Clef::Treble => self.config.treble.range = range,
                    Clef::Bass => self.config.bass.range = range,
                }
                self.config.store();
            }

            Message::ToggleSharpKeys { clef, enabled } => {
                match clef {
                    Clef::Treble => self.config.treble.sharp_keys = enabled,
                    Clef::Bass => self.config.bass.sharp_keys = enabled,
                }
                self.config.store();
            }

            _ => {}
        }

        Task::none()
    }

    pub fn view<'a>(&'a self, _: &'a App) -> Element<'a, Message> {
        let col_width = Length::Fixed(250.);
        let spacing = 20.;

        let title = {
            let label = widget::text(app::TITLE)
                .size(36)
                .font(Font::Title)
                .width(col_width)
                .align_x(alignment::Horizontal::Center);

            widget::row![
                widget::horizontal_space(),
                label,
                widget::horizontal_space()
            ]
            .width(Length::Fill)
            .align_y(alignment::Vertical::Center)
            .spacing(spacing)
        };

        let device = {
            let label = widget::text("Input device:")
                .width(col_width)
                .align_x(alignment::Horizontal::Right);

            let selector = widget::pick_list(
                &self.input_devices[..],
                Some(self.config.input_device.clone()),
                |port| Message::SelectInputPort(port),
            )
            .width(col_width);

            let btn_refresh = widget::button("Refresh").on_press(Message::RefreshDeviceList);

            widget::row![label, selector, btn_refresh]
                .width(Length::Fill)
                .align_y(alignment::Vertical::Center)
                .spacing(spacing)
        };

        const OCTAVE_SELECTION: &[OctaveRange] = &[
            OctaveRange::None,
            OctaveRange::Fixed(1),
            OctaveRange::Fixed(2),
            OctaveRange::Fixed(3),
            OctaveRange::All,
        ];

        let clef_config = |clef, label, selected, include_sharp_keys| {
            let label = widget::text(label)
                .width(col_width)
                .align_x(alignment::Horizontal::Right);

            let octave_selector =
                widget::pick_list(OCTAVE_SELECTION, Some(selected), move |range| {
                    Message::SelectOctaveRange { clef, range }
                })
                .width(col_width);

            let sharp_keys_toggle = widget::checkbox("Include sharp keys", include_sharp_keys)
                .on_toggle(move |enabled| Message::ToggleSharpKeys { clef, enabled })
                .width(col_width);

            widget::row![label, octave_selector, sharp_keys_toggle]
                .width(Length::Fill)
                .align_y(alignment::Vertical::Center)
                .spacing(spacing)
        };

        let treble_config = clef_config(
            Clef::Treble,
            "Treble octaves:",
            self.config.treble.range,
            self.config.treble.sharp_keys,
        );

        let bass_config = clef_config(
            Clef::Bass,
            "Bass octaves:",
            self.config.bass.range,
            self.config.bass.sharp_keys,
        );

        let btn_play = {
            let label = widget::text("Play")
                .size(28)
                .width(Length::Fill)
                .align_x(alignment::Horizontal::Center);

            let is_form_valid =
                !(self.config.treble.range.is_none() && self.config.bass.range.is_none());

            let btn = widget::button(label)
                .on_press_maybe(is_form_valid.then(|| {
                    Message::StateTransition(StateTransition::GameActive(self.config.clone()))
                }))
                .width(col_width);

            widget::row![widget::horizontal_space(), btn, widget::horizontal_space()]
                .width(Length::Fill)
                .align_y(alignment::Vertical::Center)
                .spacing(spacing)
        };

        let col = widget::column![
            widget::vertical_space().height(Length::FillPortion(1)),
            title,
            device,
            treble_config,
            bass_config,
            btn_play,
            widget::vertical_space().height(Length::FillPortion(3)),
        ]
        .width(Length::Fixed(790.))
        .height(Length::Fill)
        .spacing(spacing);

        widget::row![widget::horizontal_space(), col, widget::horizontal_space(),]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn subscription<'a>(&'a self, _: &'a App) -> Subscription<Message> {
        Subscription::none()
    }

    fn update_input_devices(&mut self) {
        self.input_devices = vec![input::Device::Virtual];
        self.input_devices
            .extend(input::port_list().into_iter().map(input::Device::Midi));
    }
}
