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
    std::{borrow::Cow, time::Duration},
    tap::TapFallible as _,
    wasm_bindgen::prelude::*,
    wasm_timer::Instant,
};

mod verovio;

#[wasm_bindgen(start)]
pub async fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    tracing_wasm::set_as_global_default();

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
